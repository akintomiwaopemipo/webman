use eyre::Result;
use app::{config::Node, config_db::ConfigDb};
use tokio::process::Command;
use utils::cmd::Cmd;


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
    let ssh_config_data = ssh_config.data().await?;
    
    let host = ssh_config_data.host.clone();
    
    let default_remote_dir = node.document_root().await?;

    ssh_config.write().await?;

    Cmd::run(Command::new("code").args(&[&format!("--remote=ssh-remote+{host}"), &default_remote_dir])).await?;


    Ok(())
}