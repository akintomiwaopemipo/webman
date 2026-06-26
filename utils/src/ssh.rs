use std::{path::Path, sync::Arc};

use eyre::{Result, bail};
use russh::{ChannelMsg, client, keys::{PrivateKeyWithHashAlg, PublicKey, load_secret_key}};
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
    cwd: Option<String>,
}

impl Session {

    pub async fn connect(
        host: &str,
        username: &str,
        password: &str,
        cwd: Option<&str>
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
            cwd: cwd.map(|s| s.to_string()),
        })
    }



    pub async fn connect_with_key<P: AsRef<Path>>(
        host: &str,
        username: &str,
        key_path: P,
        passphrase: Option<&str>,
        cwd: Option<&str>,
    ) -> Result<Self> {

        // Read and parse the private key from the given file path
        let key_content = std::fs::read_to_string(key_path)?;
        let key_pair = load_secret_key(&key_content, passphrase)?;

        let config = Arc::new(client::Config::default());

        // Establish connection to the remote server
        let mut session = client::connect(
            config,
            format!("{host}:22"),
            ClientHandler,
        )
        .await?;

        // 2. Query the session for the best supported signature hash (critical for RSA keys)
        let best_hash = session.best_supported_rsa_hash().await?.flatten();

        // 3. Construct the expected PrivateKeyWithHashAlg wrapper
        let auth_key = PrivateKeyWithHashAlg::new(Arc::new(key_pair), best_hash);

        // Authenticate using the corrected key signature wrapper
        if !session
            .authenticate_publickey(username, auth_key)
            .await?
            .success()
        {
            bail!("SSH key authentication failed");
        }

        Ok(Self {
            session,
            cwd: cwd.map(|s| s.to_string()),
        })
    }
    



    pub async fn exec(
        &mut self,
        command: &str
    ) -> Result<String> {

        let mut channel = self.session
            .channel_open_session()
            .await?;

        let finalized_command = match &self.cwd {
            Some(path) => format!("cd {} && {}", path, command),
            None => command.to_string(),
        };

        // Fixed: Passed ownership directly instead of using a reference &
        channel.exec(true, finalized_command).await?;

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

        // Resolve the target path based on whether cwd is provided
        let resolved_path = match &self.cwd {
            Some(path) => {
                let mut full_path = std::path::PathBuf::from(path);
                full_path.push(remote_path);
                // Convert PathBuf to a lossy String for the SFTP API
                full_path.to_string_lossy().into_owned()
            }
            None => remote_path.to_string(),
        };

        // Fixed: Use resolved_path instead of the unmapped remote_path
        let mut file = sftp
            .create(resolved_path)
            .await?;

        file.write_all(contents).await?;
        file.flush().await?;

        Ok(())
    }



    pub async fn exec_interactive(
        &mut self,
        command: &str,
    ) -> Result<String> {
        let mut channel = self.session
            .channel_open_session()
            .await?;

        channel
            .request_pty(
                true,
                "xterm-256color",
                80,
                24,
                0,
                0,
                &[],
            )
            .await?;

        let finalized_command = match &self.cwd {
            Some(path) => format!("cd {} && {}", path, command),
            None => command.to_string(),
        };

        // Fixed: Passed ownership directly instead of using a reference &
        channel.exec(true, finalized_command).await?;

        let mut output = String::new();

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } => {
                    output.push_str(&String::from_utf8_lossy(&data));
                }
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
        let mut channel = self.session.channel_open_session().await?;

        channel
            .request_pty(
                true,
                "xterm-256color",
                80,
                24,
                0,
                0,
                &[],
            )
            .await?;

        channel.request_shell(true).await?;

        if let Some(path) = &self.cwd {
            let cd_cmd = format!("cd \"{}\" && clear\n", path);
            channel.data(cd_cmd.as_bytes()).await?;
        }

        loop {
            tokio::select! {
                input = stdin_rx.recv() => {
                    match input {
                        Some(cmd) => {
                            channel.data(cmd.as_bytes()).await?;
                        }
                        None => break,
                    }
                }

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
                            break;
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
        
        let finalized_command = match &self.cwd {
            Some(path) => format!("cd {} && {}", path, command),
            None => command.to_string(),
        };

        // Fixed: Passed ownership directly instead of using a reference &
        channel.exec(true, finalized_command).await?;

        let mut local_stdout = tokio::io::stdout();

        while let Some(msg) = channel.wait().await {
            match msg {
                ChannelMsg::Data { data } | ChannelMsg::ExtendedData { data, .. } => {
                    local_stdout.write_all(&data).await?;
                    local_stdout.flush().await?;
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    if exit_status != 0 {
                        bail!("Streaming command failed with exit code {}", exit_status);
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
