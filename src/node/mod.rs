use app::config::{Config, Node};
use util::json_stringify_pretty;
use eyre::Result;

mod add;
mod edit;
mod setup;
mod root;

#[derive(clap::Args)]
pub struct Args{

    node_id: Option<String>,

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
    Root(root::Args)

}


pub async fn action(args: Args) -> Result<()> {
    
    if let Some(cmd) = args.command{
        match cmd{

            Commands::Add(args) => add::action(args).await?,
            
            Commands::Edit(args) => edit::action(args).await?,
            
            Commands::Setup(args) => setup::action(args).await?,
            
            Commands::Root(args) => root::action(args).await?,
            
        }
    }else{
        let pool = Config::connection_pool().await?;
        let node_id = args.node_id.unwrap();
        let node = Node::new(&node_id, &pool).data().await?;
        println!("{}", json_stringify_pretty(node));
    }

    Ok(())
}