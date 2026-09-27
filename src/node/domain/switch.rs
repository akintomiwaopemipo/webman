use app::{config::{Domain, Node}, config_db::ConfigDb};
use eyre::Result;



#[derive(clap::Args)]
pub struct Args{
    identifier: String
}



pub async fn action(args: Args) -> Result<()> {
    let pool = ConfigDb::connection_pool().await?;
    let node_id = Node::find(&args.identifier, &pool).await?;
    let new_domain = Domain::new(&node_id, pool).switch().await?;
    println!("Node domain switched to {new_domain}");

    Ok(())

}