use app::config::{Config, Node, Server};
use eyre::Result;
use util::shell_exec;


#[derive(clap::Args)]
pub struct Args{
    node_id: String
}


pub async fn action(args: Args) -> Result<()> {
    let pool = Config::connection_pool().await?;
    let node_id = args.node_id;
    let node = Node::new(&node_id, &pool);
    let node_data = node.data().await?;

    let server = Server::new(&node_data.host, &pool);
    let mut ssh = server.ssh().await?; 

    let ssh_username = node_data.ssh.username;
    let ssh_password = node_data.ssh.password.unwrap();
    let domain_name = node_data.domain_name;
    let bucket = node_data.backup.bucket;

    ssh.exec(&format!(r#"lemp config server-block --add-unix-user --use-certbot -u {ssh_username} -d "{domain_name}" -p "{ssh_password}""#)).await?;

    ssh.exec(&format!(r#"portal-server config server-block -u {ssh_username} -d "{domain_name}" -n {node_id}"#)).await?;

    shell_exec(&format!("webman push config {node_id}"));

    let mut ssh = node.ssh().await?;

    ssh.exec(&format!("cd public_html && portal setup --bucket '{bucket}' 2>&1")).await?;

    println!("Node successfully set up. Node Url: {}", node_data.node_url);
    println!();

    Ok(())


}