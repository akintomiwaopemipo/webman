use std::{path::{Path, PathBuf}, sync::Arc};

use eyre::{Result, bail, eyre};
use home::home_dir;
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



    pub fn default_private_key() -> Result<PathBuf> {
        let ssh_dir = home_dir()
            .ok_or_else(|| eyre!("Could not find home directory"))?
            .join(".ssh");

        for name in [
            "id_ed25519",
            "id_ecdsa",
            "id_rsa",
            "id_dsa",
        ] {
            let path = ssh_dir.join(name);
            if path.is_file() {
                return Ok(path);
            }
        }

        bail!("No default SSH private key found in {}", ssh_dir.display())
    }


    pub async fn connect_with_key<P: AsRef<Path>>(
        host: &str,
        username: &str,
        key_path: Option<P>,
        passphrase: Option<&str>,
        cwd: Option<&str>,
    ) -> Result<Self> {

        // Read and parse the private key from the given file path
        let key_path = match key_path {
            Some(path) => path.as_ref().to_path_buf(),
            None => Self::default_private_key()?,
        };
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

        // Query the session for the best supported signature hash (critical for RSA keys)
        let best_hash = session.best_supported_rsa_hash().await?.flatten();

        // Construct the expected PrivateKeyWithHashAlg wrapper
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
    


    pub async fn shell(
        &mut self,
        mut stdin_rx: Receiver<String>,
        stdout_tx: Sender<String>,
        prompt: Option<&str>, // 1. Added optional prompt customization
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

        // 2. Dynamically build the initialization shell configuration script
        let mut init_script = String::new();

        if let Some(custom_prompt) = prompt {
            init_script.push_str(&format!("export PS1='{}'\n", custom_prompt));
        }

        if let Some(path) = &self.cwd {
            init_script.push_str(&format!("cd \"{}\"\n", path));
        }

        // Clear layout screen buffer to keep custom setup commands hidden from local stdout
        if !init_script.is_empty() {
            init_script.push_str("clear\n");
            channel.data(init_script.as_bytes()).await?;
        }

        // 3. Process data loop (completed and closed out safely)
        loop {
            tokio::select! {
                input = stdin_rx.recv() => {
                    match input {
                        Some(cmd) => {
                            channel.data(cmd.as_bytes()).await?;
                        }
                        None => break, // Local loop transmitter closed
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
                            break; // Remote shell instance exited
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
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
                full_path
            }
            None => std::path::PathBuf::from(remote_path),
        };

        // Extract the parent directory and create it if it exists
        if let Some(parent) = resolved_path.parent() {
            if parent.as_os_str() != "" {
                // Note: If your SFTP crate supports recursive creation (like mkdir_p), use that.
                // Otherwise, this creates a single level.
                let _ = sftp.create_dir(parent.to_string_lossy().into_owned()).await; 
            }
        }

        // Create and write the file
        let mut file = sftp
            .create(resolved_path.to_string_lossy().into_owned())
            .await?;

        file.write_all(contents).await?;
        file.flush().await?;

        Ok(())
    }



}
