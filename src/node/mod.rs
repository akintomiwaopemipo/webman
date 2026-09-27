use app::{config::Node, config_db::ConfigDb};
use util::json_stringify_pretty;
use eyre::{Result, eyre};

mod add;
mod edit;
mod setup;
mod root;
mod domain;


#[derive(clap::Args)]
pub struct Args{

    identifier: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands{

    #[command(about = "Add a new node")]
    Add(add::Args),

    #[command(about = "Edit node")]
    Edit(edit::Args),

    #[command(about = "Setup a new node")]
    Setup(setup::Args),

    #[command(about = "Get root of node")]
    Root(root::Args),

    #[command(about = "Manage node domain")]
    Domain(domain::Args),

}


pub async fn action(args: Args) -> Result<()> {
    
    if let Some(cmd) = args.command{
        match cmd{

            Commands::Add(args) => add::action(args).await?,
            
            Commands::Edit(args) => edit::action(args).await?,
            
            Commands::Setup(args) => setup::action(args).await?,
            
            Commands::Root(args) => root::action(args).await?,

            Commands::Domain(args) => domain::action(args.command.unwrap()).await?,
            
        }
    }else{
        let pool = ConfigDb::connection_pool().await?;
        let identifier = args.identifier.ok_or_else(|| eyre!("Identifier is required"))?;
        let node_id = Node::find(&identifier, &pool).await?;
        let node = Node::new(&node_id, &pool).data().await?;
        println!("{}", json_stringify_pretty(node));
    }

    Ok(())
}