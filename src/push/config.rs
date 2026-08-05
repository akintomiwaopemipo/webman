use app::{config::{Config, Node}, config_db::ConfigDb};
use eyre::Result;

#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "NodeId to push receive config")]
    identifier: Option<String>,

    #[arg(short, long, help = "Upload config to all active nodes")]
    all: bool
}


pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;
    
    let push_to_all_nodes = args.all;

    if let Some(identifier) = args.identifier{
        let node_id = Node::find(&identifier, &pool).await?;
        Node::new(&node_id, &pool).push().await?;

    }else if push_to_all_nodes {
        let config = Config::new(&pool);
        for node_id in config.active_node_ids().await? {
            let node = Node::new(&node_id, &pool).data().await?;

            println!();
            println!("### ----------------------------------------------------------");
            println!("###               {} ({})", node.name, node_id);
            println!("### ----------------------------------------------------------");

            Node::new(&node_id, &pool).push().await?;

        }
    }

    Ok(())
}
