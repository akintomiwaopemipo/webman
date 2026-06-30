use eyre::{Result, eyre};
use app::{config::Node, config_db::ConfigDb};
use prelude::PathExt;
use tokio::{fs, process::Command};
use utils::cmd::Cmd;
use dirs::data_local_dir;


#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "The node ID of the node, also the hostname of the SSH config")]
    node_id: Option<String>,

    #[arg(short, long, help = "Verbose output")]
    verbose: bool
}



pub async fn action(args: Args) -> Result<()> {
    
    let pool = ConfigDb::connection_pool().await?;

    let node_id = args.node_id.unwrap();
    let node = Node::new(&node_id, &pool);
    let ssh_config = node.ssh_config().await?;
    
    let host = ssh_config.host;
    let user = ssh_config.user;
    let host_name = ssh_config.host_name;
    let identity_file = ssh_config.identity_file;
    
    let default_remote_dir = node.document_root().await?;


    if !node.ssh().await.is_ok() {
        Cmd::exec(Command::new("webman").args(&["push", "pbk", &node_id])).await?;
    }


    let vsode_temp_dir = data_local_dir().ok_or_else(|| eyre!("Could not determine local data directory"))?.join("webman");
    let vscode_user_dir = vsode_temp_dir.join("User");
    let vscode_ssh_config_file = vscode_user_dir.join("ssh-config");

    if args.verbose {
        println!("VSCode temp dir: {}", vsode_temp_dir.as_string());
        println!("VSCode user dir: {}", vscode_user_dir.as_string());
        println!("VSCode SSH config file: {}", vscode_ssh_config_file.as_string());
    }
    fs::create_dir_all(&vscode_user_dir).await?;


    fs::write(&vscode_ssh_config_file, format!("Host {host}\n\tHostName {host_name}\n\tUser {user}\n\tIdentityFile {identity_file}\n\tIdentitiesOnly yes\n")).await?;

    fs::write(vscode_user_dir.join("settings.json"), &format!(r#"{{"remote.SSH.configFile": "{}"}}"#, vscode_ssh_config_file.display())).await?;

    Cmd::run(Command::new("code").args(&[
        "--user-data-dir", &vsode_temp_dir.as_string(),
        &format!("--remote=ssh-remote+{host}"),
        &default_remote_dir
    ])).await?;


    Ok(())
}