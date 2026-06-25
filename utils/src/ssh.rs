use std::sync::Arc;

use eyre::{Result, bail};
use russh::{ChannelMsg, client, keys::PublicKey};
use russh_sftp::client::SftpSession;
use tokio::io::AsyncWriteExt;

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
}