use eyre::Result;

mod run;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

}


#[derive(clap::Subcommand)]
pub enum Commands {

    #[command(about = "Run sqlx migrations")]
    Run(run::Args)

}




pub async fn action(args: Args) -> Result<()> {

    if let Some(cmd) = args.command{
        match cmd{

            Commands::Run(args) => run::action(args).await?,
    
        }
    }

    Ok(())
}