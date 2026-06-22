use app::config::Config;
use eyre::Result;
use db::generate_models::generate_models;

#[derive(clap::Args)]
pub struct Args;


pub async fn action(_args: Args) -> Result<()> {

    let pool = Config::connection_pool().await?;

    generate_models(&pool).await?;

    println!("Models generated successfully.");

    Ok(())

}
