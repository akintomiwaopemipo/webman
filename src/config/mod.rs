use app::config::Config;
use eyre::Result;
use prelude::SerdeJsonSerialize;
mod virtual_host;

#[derive(clap::Args)]
pub struct Args{
    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands{

    #[command(about = "Config virtual host of remote node")]
    VirtualHost(virtual_host::Args)

}


pub async fn action(args: Args) -> Result<()> {

    if let Some(cmd) = args.command {
        match cmd {
            
            Commands::VirtualHost(args) => virtual_host::action(args).await?

        }
    } else {
        let pool = Config::connection_pool().await?;
        Config::new(&pool).data().await?.print();
    }

    Ok(())
}