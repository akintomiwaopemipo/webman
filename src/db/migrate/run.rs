use app::config::Config;
use eyre::Result;

#[derive(clap::Args)]
pub struct Args;


pub async fn action(_args: Args) -> Result<()> {
    println!("Running sqlx migrations...");
    
    let pool = Config::connection_pool().await?;

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Error running migrations: {e}");
    } else {
        println!("Migrations ran successfully.");
    }

    Ok(())

}