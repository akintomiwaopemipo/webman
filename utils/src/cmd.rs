use std::process::Stdio;

use eyre::{Result, bail, eyre};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, process::Command};


pub struct Cmd;

impl Cmd {

    pub async fn run(cmd: &mut Command) -> Result<String> {
        let output = cmd.output().await?;

        if !output.status.success() {
            bail!(
                "command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

        Ok(stdout.trim().to_string())
    }



    pub async fn exec(cmd: &mut Command) -> Result<String> {
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| eyre!("Failed to capture stdout"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| eyre!("Failed to capture stderr"))?;

        let stdout_task = tokio::spawn(async move {
            let mut reader = stdout;
            let mut terminal = tokio::io::stdout();
            let mut captured = Vec::new();
            let mut buf = [0u8; 8192];

            loop {
                let n = reader.read(&mut buf).await?;

                if n == 0 {
                    break;
                }

                terminal.write_all(&buf[..n]).await?;
                terminal.flush().await?;

                captured.extend_from_slice(&buf[..n]);
            }

            Ok::<Vec<u8>, eyre::Error>(captured)
        });

        let stderr_task = tokio::spawn(async move {
            let mut reader = stderr;
            let mut terminal = tokio::io::stderr();
            let mut buf = [0u8; 8192];

            loop {
                let n = reader.read(&mut buf).await?;

                if n == 0 {
                    break;
                }

                terminal.write_all(&buf[..n]).await?;
                terminal.flush().await?;
            }

            Ok::<(), eyre::Error>(())
        });

        let status = child.wait().await?;

        let stdout_bytes = stdout_task.await??;
        stderr_task.await??;

        if !status.success() {
            bail!("command failed with status {status}");
        }

        Ok(String::from_utf8_lossy(&stdout_bytes).into_owned())
    }



    pub async fn pipe(left: &mut Command, right: &mut Command) -> Result<()> {
        let mut left_child = left
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let left_stdout: Stdio = left_child
            .stdout
            .take()
            .ok_or_else(|| eyre!("Failed to capture left stdout"))?
            .try_into()?;

        let mut right_child = right
            .stdin(left_stdout)
            .stderr(Stdio::inherit())
            .spawn()?;

        let right_status = right_child.wait().await?;
        let left_status = left_child.wait().await?;

        if !left_status.success() {
            let left_program = left.as_std().get_program().to_string_lossy().into_owned();
            bail!("left command ({left_program}) failed with status {left_status}");
        }

        if !right_status.success() {
            let right_program = right.as_std().get_program().to_string_lossy().into_owned();
            bail!("right command ({right_program}) failed with status {right_status}");
        }

        Ok(())
    }

}