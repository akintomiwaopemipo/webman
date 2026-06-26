use std::sync::Arc;

use eyre::{Result, bail};
use russh::{ChannelMsg, client, keys::PublicKey};
use russh_sftp::client::SftpSession;
use tokio::{io::AsyncWriteExt, sync::mpsc::{Receiver, Sender}};

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct Session {
    session: client::Handle<ClientHandler>,
}

impl Session {

    pub async fn connect(
        host: &str,
        username: &str,
        password: &str,
    ) -> Result<Self> {

        let config = Arc::new(client::Config::default());

        let mut session = client::connect(
            config,
            format!("{host}:22"),
            ClientHandler,
        )
        .await?;

        if !session
            .authenticate_password(username, password)
            .await?
            .success()
        {
            bail!("SSH authentication failed");
        }

        Ok(Self {
            session,
        })
    }

    pub async fn exec(
        &mut self,
        command: &str,
    ) -> Result<String> {

        let mut channel = self.session
            .channel_open_session()
            .await?;

        channel.exec(true, command).await?;

        let mut output = String::new();

        while let Some(msg) = channel.wait().await {

            match msg {

                ChannelMsg::Data { data } => {
                    output.push_str(
                        &String::from_utf8_lossy(&data)
                    );
                }

                ChannelMsg::ExtendedData { data, .. } => {
                    output.push_str(
                        &String::from_utf8_lossy(&data)
                    );
                }

                ChannelMsg::ExitStatus { exit_status } => {

                    if exit_status != 0 {

                        bail!(
                            "Command failed with exit code {}:\n{}",
                            exit_status,
                            output
                        );
                    }
                }

                _ => {}
            }
        }

        Ok(output)
    }

    
    
    pub async fn upload(
        &mut self,
        remote_path: &str,
        contents: &[u8],
    ) -> Result<()> {

        let len = contents.len();
        println!("local({len} bytes) -> {remote_path}");

        let channel = self.session
            .channel_open_session()
            .await?;

        channel
            .request_subsystem(true, "sftp")
            .await?;


        let sftp = SftpSession::new(
            channel.into_stream()
        )
        .await?;

        let mut file = sftp
            .create(remote_path)
            .await?;

        file.write_all(contents).await?;
        file.flush().await?;

        Ok(())
    }




    pub async fn exec_interactive(
        &mut self,
        command: &str,
    ) -> Result<String> {
        // 1. Open a standard session channel
        let mut channel = self.session
            .channel_open_session()
            .await?;

        // 2. Request a pseudo-terminal (PTY) to force terminal-style output formatting
        channel
            .request_pty(
                true,
                "xterm-256color", // Enables ANSI color codes
                80,               // Standard terminal column width
                24,               // Standard terminal row height
                0,
                0,
                &[],
            )
            .await?;

        // 3. Execute the single command under the PTY context
        channel.exec(true, command).await?;

        let mut output = String::new();

        // 4. Stream data until the remote processes exits
        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    output.push_str(&String::from_utf8_lossy(&data));
                }
                // Under a PTY, stderr is often multiplexed into stdout (ChannelMsg::Data)
                ChannelMsg::ExtendedData { data, .. } => {
                    output.push_str(&String::from_utf8_lossy(&data));
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    if exit_status != 0 {
                        bail!(
                            "Interactive command failed with exit code {}:\n{}",
                            exit_status,
                            output
                        );
                    }
                }
                _ => {}
            }
        }

        Ok(output)
    }




    pub async fn shell(
        &mut self,
        mut stdin_rx: Receiver<String>,
        stdout_tx: Sender<String>,
    ) -> Result<()> {
        // 1. Open a new session channel
        let mut channel = self.session.channel_open_session().await?;

        // 2. Request a pseudo-terminal (PTY) to emulate a real terminal screen
        channel
            .request_pty(
                true,
                "xterm-256color", // Terminal type emulation
                80,               // Terminal width in characters
                24,               // Terminal height in characters
                0,                // Pixel width
                0,                // Pixel height
                &[],              // Custom terminal modes (empty defaults)
            )
            .await?;

        // 3. Start the interactive shell subsystem
        channel.request_shell(true).await?;

        // 4. Split the channel into reading and writing tasks
        loop {
            tokio::select! {
                // Handle user inputs sent to stdin_rx
                input = stdin_rx.recv() => {
                    match input {
                        Some(cmd) => {
                            // Write command bytes to the remote shell
                            channel.data(cmd.as_bytes()).await?;
                        }
                        None => break, // Channel closed by local caller
                    }
                }

                // Handle incoming data streams from the remote server
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { data }) => {
                            let text = String::from_utf8_lossy(&data).into_owned();
                            if stdout_tx.send(text).await.is_err() {
                                break;
                            }
                        }
                        Some(ChannelMsg::ExtendedData { data, .. }) => {
                            let text = String::from_utf8_lossy(&data).into_owned();
                            if stdout_tx.send(text).await.is_err() {
                                break;
                            }
                        }
                        Some(ChannelMsg::ExitStatus { .. }) | None => {
                            break; // Remote shell exited
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }




    pub async fn exec_stream_to_stdout(
        &mut self,
        command: &str,
    ) -> Result<()> {
        let mut channel = self.session.channel_open_session().await?;

        channel.request_pty(true, "xterm-256color", 80, 24, 0, 0, &[]).await?;
        channel.exec(true, command).await?;

        // Use standard output handle for unbuffered raw byte printing
        let mut local_stdout = tokio::io::stdout();

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } | ChannelMsg::ExtendedData { data, .. } => {
                    // Flush bytes immediately to the screen as they arrive
                    local_stdout.write_all(&data).await?;
                    local_stdout.flush().await?;
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    if exit_status != 0 {
                        bail!("Command exited with status {exit_status}");
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }



    pub async fn exec_stream_to_channel(
        &mut self,
        command: &str,
        tx: Sender<String>,
    ) -> Result<()> {
        let mut channel = self.session.channel_open_session().await?;

        channel.request_pty(true, "xterm-256color", 80, 24, 0, 0, &[]).await?;
        channel.exec(true, command).await?;

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } | ChannelMsg::ExtendedData { data, .. } => {
                    let text = String::from_utf8_lossy(&data).into_owned();
                    // Send text chunk immediately to the receiver
                    if tx.send(text).await.is_err() {
                        break; // Receiver disconnected
                    }
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    if exit_status != 0 {
                        bail!("Command exited with status {exit_status}");
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }




}