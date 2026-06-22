use eyre::Result;
mod clone;


#[derive(clap::Args)]
#[command(arg_required_else_help(true))]
pub struct Args{
    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands{

    #[command(about = "Clone given node to beta")]
    Clone(clone::Args)

}


pub async fn action(cmd: Commands) -> Result<()> {

    match cmd {
        
        Commands::Clone(args) => clone::action(args).await?
       
    }
    Ok(())
}