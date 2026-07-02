use app::{config::Node, config_db::ConfigDb};
use eyre::Result;
use util::shell_exec;


#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "The node ID of the node, also the hostname of the SSH config")]
    node_id: Option<String>,

    #[arg(long)]
    preflight: bool,
}



pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;    
    
    println!();

    let node_id = args.node_id.unwrap();
    let node = Node::new(&node_id, &pool);
    let ssh_config = node.ssh_config().await?;
    let ssh_config_data = ssh_config.data().await?;
    
    let host = ssh_config_data.host.clone();

    ssh_config.write().await?;

    if args.preflight {
        ssh_config.preflight().await?;
    }
    
    let home = node.document_root().await?;

    shell_exec(&format!(r#"ssh -o ServerAliveInterval=300 -t {host} "cd {home} ; bash --login" "#));

    Ok(())
}