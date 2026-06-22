use eyre::Result;

mod migrate;
mod generate_models;


#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

}


#[derive(clap::Subcommand)]
pub enum Commands {

    #[command(about = "Run sqlx migrations")]
    Migrate(migrate::Args),

    #[command(about = "Generate models from the database schema")]
    GenerateModels(generate_models::Args),

}




pub async fn action(args: Args) -> Result<()> {

    if let Some(cmd) = args.command{
        match cmd{

            Commands::GenerateModels(args) => generate_models::action(args).await?,

            Commands::Migrate(args) => migrate::action(args).await?,
            
        }
    }

    Ok(())
}