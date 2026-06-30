use std::path::Path;

use tokio::fs;
use eyre::Result;

pub async fn add_or_update_host<P: AsRef<Path>>(
    config_path: P,
    host: &str,
    hostname: &str,
    user: &str,
    identity_file: &str,
) -> Result<()> {

    let path = config_path.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    

    let content = if path.exists() {
        fs::read_to_string(path).await?
    } else {
        String::new()
    };

    let new_block = format!(
        "Host {host}\n\
         \tHostName {hostname}\n\
         \tUser {user}\n\
         \tIdentityFile {identity_file}\n\
         \tIdentitiesOnly yes\n"
    );

    let mut output = String::new();
    let mut current = String::new();
    let mut replaced = false;

    for line in content.lines() {
        if line.trim_start().starts_with("Host ") {
            if !current.is_empty() {
                if is_host_block(&current, host) {
                    output.push_str(&new_block);
                    output.push('\n');
                    replaced = true;
                } else {
                    output.push_str(&current);
                }
                current.clear();
            }
        }

        current.push_str(line);
        current.push('\n');
    }

    if !current.is_empty() {
        if is_host_block(&current, host) {
            output.push_str(&new_block);
            output.push('\n');
            replaced = true;
        } else {
            output.push_str(&current);
        }
    }

    if !replaced {
        if !output.ends_with('\n') {
            output.push('\n');
        }
        output.push('\n');
        output.push_str(&new_block);
    }

    fs::write(path, output).await?;
    Ok(())
}

fn is_host_block(block: &str, host: &str) -> bool {
    block
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if line.starts_with("Host ") {
                Some(line["Host ".len()..].trim())
            } else {
                None
            }
        })
        .map(|value| value == host)
        .unwrap_or(false)
}