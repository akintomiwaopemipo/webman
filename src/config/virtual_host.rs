use app::config::{Config, Node, Server};
use eyre::Result;
use sqlx::SqlitePool;

#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "NodeId of node for virtual host config")]
    node_id: Option<String>,

    #[arg(short, long, help = "Config all nodes")]
    all: bool,

    #[arg(long)]
    dry_run: bool
}


pub async fn action(args: Args) -> Result<()> {

    let pool = Config::connection_pool().await?;

    let push_to_all_nodes = args.all;

    if let Some(node_id) = args.node_id{

        config_virtual_host(&node_id, args.dry_run, &pool).await?;

    }else if push_to_all_nodes{
        for node_id in Config::new(&pool).active_node_ids().await? {
            let node = Node::new(&node_id, &pool).data().await?;

            println!();
            println!("### ----------------------------------------------------------");
            println!("###               {} ({})", node.name, node_id);
            println!("### ----------------------------------------------------------");

            config_virtual_host(&node_id, args.dry_run, &pool).await?;

        }
    }

    Ok(())
}


async fn config_virtual_host(node_id: &str, dry_run: bool, pool: &SqlitePool) -> Result<()> {
    
    let node = Node::new(node_id, pool).data().await?;

    let mut ssh = Server::new(&node.host, &pool).ssh().await?;

    let ssh_username = node.ssh.username;
    let domain_name = node.domain_name;

    let command = format!(r#"wpanel config virtual-host -u "{ssh_username}" -d "{domain_name}" -n "{node_id}" --use-certbot"#);

    if dry_run{
        println!("{command}");
    }else{
        ssh.exec(&command).await?;
    }

    println!();

    Ok(())
}