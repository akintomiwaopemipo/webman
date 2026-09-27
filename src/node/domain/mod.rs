use eyre::Result;

mod switch;

#[derive(clap::Args)]
#[command(arg_required_else_help(true))]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands {
    
    #[command(about = "Switch node domain between app domain and custom domain")]
    Switch(switch::Args)

}


pub async fn action(cmd: Commands) -> Result<()> {
    
    match cmd{

        Commands::Switch(args) => switch::action(args).await?,
    }

    Ok(())
}


