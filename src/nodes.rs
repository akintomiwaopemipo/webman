use app::{config::Config, config_db::ConfigDb};
use cli_table::{print_stdout, Table};
use prelude::SerdeJsonSerialize;
use eyre::Result;


#[derive(clap::Args)]
pub struct Args{

    #[arg(long)]
    dns: bool,

    #[arg(long)]
    active: bool,

    #[arg(long)]
    table: bool,

    #[arg(long)]
    json: bool
}


pub async fn action(args: Args) -> Result<()> {
    let pool = ConfigDb::connection_pool().await?;
    let nodes = if args.active{
        Config::new(&pool).nodes().await?
            .into_iter()
            .filter(|(_, node)| node.active)
            .collect()
    }else{
        Config::new(&pool).nodes().await?
    };

    
    if args.json{
        println!("{}", nodes.stringify_pretty());
    } else if args.dns {
        
        for node in nodes.into_values(){
            println!("{domain} -> {ip}", domain = node.domain_name, ip = node.host);
        }

    } else if args.table {

        let mut _table = Vec::<Vec<String>>::new();

        for (index, (node_id, node)) in nodes.into_iter().enumerate(){
            _table.push(vec![
                (index + 1).to_string(),
                node_id.to_owned(),
                node.app_id,
                node.name,
                node.host,
                node.node_url,
                node.custom_domain.map_or_else(|| "None".to_string(), |cd| format!("https://{cd}"))
            ]);
        }

        println!();
        print_stdout(_table.clone().table().title(vec![
            "S/N",
            "Node ID",
            "App ID",
            "Name",
            "Host",
            "Node Url",
            "Custom Domain"
        ])).unwrap();
        println!();

    } else {

        for (node_id, node) in nodes{
            println!("Node Id: {node_id}, Name: {name}, Host: {host}, Node Url: {node_url}, Custom Domain: {custom_domain}",
                name = node.name,
                host = node.host,
                node_url = node.node_url,
                custom_domain = node.custom_domain.map_or_else(|| "None".to_string(), |cd| format!("https://{cd}"))
            );
        }

    }

    Ok(())
}