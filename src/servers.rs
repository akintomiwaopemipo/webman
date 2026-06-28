use app::{config::Config, config_db::ConfigDb};
use prelude::SerdeJsonSerialize;
use eyre::Result;


#[derive(clap::Args)]
pub struct Args{

    #[arg(long)]
    json: bool
}


pub async fn action(args: Args) -> Result<()> {
    let pool = ConfigDb::connection_pool().await?;
    let roots = Config::new(&pool).data().await?.servers;

    
    if args.json {
        println!("{}", roots.stringify_pretty());
    } else {

        for (root_ip, server) in roots{
            let hostname = server.hostname;
            println!("{hostname}, IP: {root_ip}");
        }

        println!();
    }

    Ok(())
}