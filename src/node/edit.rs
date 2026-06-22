use app::{app::App, config::{Node, NodeData}, config_db::ConfigDb};
use prelude::SerdeJsonSerialize;
use util::{file_get_contents, file_put_contents, shell_exec};
use eyre::Result;


#[derive(clap::Args)]
pub struct Args{
    node_id: String
}


pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;
    let node_id = args.node_id;
    let node = Node::new(&node_id, &pool);
    let node_data = node.data().await?;


    let tmp_file = App::new_tmp_file("json", 7);
    let new_tmp_file = App::new_tmp_file("json", 7);

    let mut node_string = node_data.stringify_pretty();
    node_string.push_str("\n");

    file_put_contents(&tmp_file, &node_string);
    file_put_contents(&new_tmp_file, &node_string);

    shell_exec(&format!(r#"nano "{new_tmp_file}""#));

    let file_content = file_get_contents(&new_tmp_file);

    let new_node = serde_json::from_str::<NodeData>(&file_content).unwrap();

    Node::update(new_node, &pool).await?;

    shell_exec(&format!(r#"git diff --no-index "{tmp_file}" "{new_tmp_file}" "#));

    std::fs::remove_file(&tmp_file).unwrap();
    std::fs::remove_file(&new_tmp_file).unwrap();

    Ok(())
}