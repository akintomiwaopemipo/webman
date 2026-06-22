use eyre::Result;

mod update;
mod generate;

#[derive(clap::Args)]
#[command(arg_required_else_help(true))]
pub struct Args{
    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands{

    #[command(about = "Update SSH config")]
    Update(update::Args),

    #[command(about = "Generate SSH config")]
    Generate(generate::Args),

}


pub async fn action(cmd: Commands) -> Result<()> {
    match cmd{

        Commands::Update(args) => update::action(args).await,

        Commands::Generate(args) => generate::action(args).await?
    }

    Ok(())
}