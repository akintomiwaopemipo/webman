use app::{app::App, config::{Config, Node}, config_db::ConfigDb};
use sqlx::SqlitePool;
use tokio::fs;
use util::file_put_contents;
use serde_json::json;
use eyre::Result;

#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "NodeId to push receive config")]
    node_id: Option<String>,

    #[arg(short, long, help = "Upload config to all active nodes")]
    all: bool
}


pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;
    
    let push_to_all_nodes = args.all;

    if let Some(node_id) = args.node_id{

        push_config(&node_id, &pool).await?;

    }else if push_to_all_nodes {
        let config = Config::new(&pool);
        for node_id in config.active_node_ids().await? {
            let node = Node::new(&node_id, &pool).data().await?;

            println!();
            println!("### ----------------------------------------------------------");
            println!("###               {} ({})", node.name, node_id);
            println!("### ----------------------------------------------------------");

            push_config(&node_id, &pool).await?;

        }
    }

    Ok(())
}


async fn push_config(node_id: &str, pool: &SqlitePool) -> Result<()> {
    let config = Config::new(pool).data().await?;

    let node = Node::new(node_id, &pool);
    let node_data = node.data().await?;


    let tmp_file = App::new_tmp_file("json", 7);

    let mut _node  =  json!({
        "node_id": node_id
    });

    util::object_assign(&mut _node , serde_json::from_str(&util::json_stringify(node_data.clone())).unwrap());

    util::file_put_contents(&tmp_file, &util::json_stringify(_node));

    let mut ssh = node.ssh().await?;

    let remote_path = format!("{}/{}.json", node.document_root().await?, config.nodes_config_name);
    ssh.upload(&remote_path, &fs::read(&tmp_file).await?).await?;

    let mut cnf_tmp: Option<String> = None;

    let mysql_opt = node_data.mysql.clone();

    if mysql_opt.is_some(){
        let mysql = mysql_opt.unwrap();
        cnf_tmp = Some(App::new_tmp_file("json", 7));
        file_put_contents(&cnf_tmp.clone().unwrap(), &format!("[client]\nuser = {}\npassword = {}\n", mysql.username.unwrap_or(node_data.ssh.username.clone()), mysql.password.unwrap_or(node_data.ssh.password.clone().unwrap_or_default())));

        ssh.upload(
            &format!("{}/.my.cnf", node.remote_dir().await?),
            &fs::read(&cnf_tmp.clone().unwrap()).await?
        ).await?;
    }


    std::fs::remove_file(tmp_file).unwrap();
    if cnf_tmp.is_some(){
        std::fs::remove_file(cnf_tmp.unwrap()).unwrap();
    }

    println!();

    Ok(())
}