use app::config::{Config, Node};
use eyre::Result;


#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "NodeId of node to be cloned to beta")]
    node_id: Option<String>,

    #[arg(short = 'x', help = "Extract upload specs directory")]
    extract: bool,

    #[arg(long, help = "Clone only databases only")]
    db_only: bool,

    #[arg(long, help = "Clone node from source")]
    from_source: bool
}




pub async  fn action(args: Args) -> Result<()> {

    let pool = Config::connection_pool().await?;
    let node_id = args.node_id.unwrap();
    let node = Node::new(&node_id, &pool);
    let node_data = node.data().await?;
    let beta = Node::new("template", &pool);
    let beta_data = beta.data().await?;


    if !args.from_source{

        println!("Connecting to {}", beta_data.name);

        match beta.ssh().await {
            Ok(mut ssh) => {

                println!("Connected!");

                ssh.exec(&format!(r#"cd public_html && portal backup install --bucket "{}" "#, node_data.backup.bucket)).await?;
            },
            Err(_) => println!("Could not connect to node beta")
        }

    }

    Ok(())
}