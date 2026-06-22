use app::{config::{Node, Server}, config_db::ConfigDb};
use eyre::Result;
use util::shell_exec;


#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "The node ID of the node, also the hostname of the SSH config")]
    node_id: Option<String>,

    #[arg(long, help = "Indicate remote dev for hostname, --ip required")]
    hostname: bool,

    #[arg(long, help = "IP Address of the root node")]
    ip: Option<String>,

    #[arg(short = 'z', long, help = "IP Address of root in config")]
    root_ip: Option<String>
}



pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;    
    
    println!();

    if let Some(node_id) = args.node_id{

        let node = Node::new(&node_id, &pool);
        let node_data = node.data().await?;

        if !node.ssh().await.is_ok() { shell_exec(&format!("webman push pbk {node_id}")) }

        shell_exec(
            &format!(
                r#"ssh -o ServerAliveInterval=300 -t {username}@{host} "cd {home} ; bash --login" "#, username = node_data.ssh.username, host = node_data.host, home = node_data.home.unwrap_or(format!("/home/{username}", username = node_data.ssh.username))
            )
        );

    }else if let Some(root_ip) = args.root_ip{

        let server = Server::new(&root_ip, &pool);

        if !server.ssh().await.is_ok() { shell_exec(&format!("webman push pbk --root-ip {root_ip}")) }
        
        shell_exec(
            &format!(r#"ssh -o ServerAliveInterval=300 -t root@{root_ip} "cd / ; bash --login" "#)
        );

    }

    Ok(())
}