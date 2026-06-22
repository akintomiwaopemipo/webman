use eyre::Result;

mod pbk;
mod config;

#[derive(clap::Args)]
#[command(arg_required_else_help(true))]
pub struct Args{
    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands{

    #[command(about = "Upload public key to nodes using username and password")]
    Pbk(pbk::Args),

    #[command(about = "Upload config file to node")]
    Config(config::Args)

}


pub async fn action(cmd: Commands) -> Result<()> {

    match cmd {
        
        Commands::Pbk(args) => pbk::action(args).await?,

        Commands::Config(args) => config::action(args).await?
    }

    Ok(())
}