use eyre::Result;
use app::{config::{Config, Node, NodeBackup, NodeData, NodeSsh, Server}, config_db::ConfigDb};
use colored::Colorize;
use eyre::Ok;
use util::{shell_exec, shell_exec_to_string, stdin, stdin_or_default};



#[derive(clap::Args)]
pub struct Args{
    #[arg(long)]
    no_dns_check: bool
}


pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;
    let config = Config::new(&pool);
    let node_id: String;

    loop{
        let random_hex = util::random_hex(8);
        if random_hex.chars().nth(0).unwrap().is_alphabetic(){
            if !config.node_ids().await?.contains(&random_hex){
                node_id = random_hex;
                break;
            }
        }
    }


    let mut config_data = config.data().await?;

    // let last_node = config.nodes.clone().into_iter().last().unwrap().1;

    let app_id = stdin("App ID: ");
    let domain_name = stdin_or_default("Domain name: ", &format!("{app_id}.icitifysms.com"));
    let resolved_host_output= shell_exec_to_string(&format!("dig +short '{domain_name}' | head -n 1"));
    let mut resolved_host= resolved_host_output.trim();
    if !args.no_dns_check{
        if resolved_host.is_empty(){
            println!("{}", format!("Could not resolve domain: {domain_name}").bright_red());
            println!();
            std::process::exit(1);
        }
    }else{
        resolved_host = "";
    }
    
    let name = util::stdin_or_default("Name", &domain_name);
    let host = util::stdin_or_default("Host", &resolved_host);
    let ssh_username = util::stdin_or_default("SSH Username", &domain_name);
    let ssh_password = util::random_characters(21);
    let home = format!("/home/{ssh_username}/public_html");


    

    config_data.nodes.insert(node_id.clone(), NodeData {
        node_id: node_id.clone(),
        name,
        app_id,
        domain_name: domain_name.clone(),
        custom_domain: None,
        host,
        base_url: Some(format!("https://{domain_name}")),
        rel_dirname: Some("".to_string()),
        node_url: format!("https://{domain_name}"),
        home: Some(home),
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
    });


    let (node_id, node) = config_data.nodes.clone().into_iter().last().unwrap();

    println!();
    println!("Node Id: {node_id}");
    println!("{}", util::json_stringify_pretty(node.clone()));
    println!();

    if stdin("Do you want to continue with the above configuration: ").to_lowercase() != "y"{
        println!();
        std::process::exit(0);
    }


    Node::add(node.clone(), &pool).await?;
    
    let mut ssh = Server::new(&node.host, &pool).ssh().await?;

    
    let ssh_username = node.ssh.username;
    let ssh_password = node.ssh.password.unwrap();
    let domain_name = node.domain_name;

    ssh.exec(&format!(r#"lemp config server-block --add-unix-user --use-certbot -u {ssh_username} -d "{domain_name}" -p "{ssh_password}""#)).await?;

    ssh.exec(&format!(r#"portal-server config server-block -u {ssh_username} -d "{domain_name}" -n {node_id}"#)).await?;

    shell_exec(&format!("webman push config {node_id}"));

    ssh.exec(&format!(r#"systemctl restart portal-webserver && echo "Restarted Portal Webserver" "#)).await?;

    let mut ssh = Node::new(&node_id, &pool).ssh().await?;

    ssh.exec("cd public_html && portal setup 2>&1").await?;

    println!("Node successfully added. Node Url: {}", node.node_url);
    println!();


    Ok(())


}