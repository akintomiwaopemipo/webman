use db::entities::Nodes;
use eyre::Result;
use app::{config::{Node, NodeBackup, NodeData, NodeSsh, Server}, config_db::ConfigDb};
use eyre::Ok;
use prelude::SerdeJsonSerialize;
use tokio::process::Command;
use util::{stdin, stdin_or_default};
use utils::{cmd::Cmd, random_characters};



#[derive(clap::Args)]
pub struct Args;


pub async fn action(_args: Args) -> Result<()> {

    Cmd::exec(Command::new("git").arg("pull")).await?;

    let pool = ConfigDb::connection_pool().await?;

    let node_id = db::unique_hex(Nodes::Table, Nodes::NodeId, 8, &pool).await;
    let app_id = stdin("App ID: ");
    let domain_name = stdin_or_default("Domain name: ", &format!("{app_id}.icitifysms.com"));
    let host = stdin("Host (IP Address): ");
    let name = stdin_or_default("Name", &app_id);
    
    let home = {
        let home = stdin("Home: ");
        let home = home.trim().to_owned();
        (!home.is_empty()).then_some(home)
    };
    
    let ssh_username = stdin("SSH Username: ");
    let mut ssh_password = "".to_string();

    if !ssh_username.trim().is_empty(){
        ssh_password = random_characters(21);     
    }
   

    let node_data = NodeData {
        node_id: node_id.clone(),
        name: name.clone(),
        app_id: app_id.clone(),
        domain_name: domain_name.clone(),
        custom_domain: None,
        host,
        base_url: Some(format!("https://{domain_name}")),
        rel_dirname: Some("".to_string()),
        node_url: format!("https://{domain_name}"),
        home,
        hostname: None,
        remote_home_dir: None,
        mysql: None,
        ssh: NodeSsh {
            username: ssh_username,
            password: Some(ssh_password),
            private_key: None
        },
        backup: NodeBackup {
            bucket: domain_name,
            regulation_range: 10
        },
        timezone_offset: None,
        mimics: None,
        dev_mode: false,
        active: true
    };


    println!();
    println!("Node Id: {node_id}");
    node_data.pretty_print();
    println!();

    if stdin("Do you want to continue with the above configuration: ").to_lowercase() != "y"{
        println!();
        std::process::exit(0);
    }


    Node::add(node_data.clone(), &pool).await?;

    let node = Node::new(&node_id, &pool);

    Cmd::exec(Command::new("git").args(&["commit", "-m", &format!("Add node {node_id}, app_id: {app_id}, name: {name}")])).await?;

    Cmd::exec(Command::new("git").arg("push")).await?;

    node.push().await?;
    
    let mut ssh = node.ssh().await?;
    let mut central_server_ssh = Server::central_server(&pool).await?.ssh().await?;

    central_server_ssh.exec_stream_to_stdout(&format!("icitifysms-central node legacy-php configure {node_id}")).await?;

    ssh.exec(&format!(r#"systemctl restart icitifysms-webserver && echo "Restarted Icitifysms Webserver" "#)).await?;

     ssh.exec("icitifysms setup 2>&1").await?;

    println!("Node successfully added. Node Url: {}", node_data.node_url);
    println!();


    Ok(())


}