use eyre::Result;

use app::{config::Node, config_db::ConfigDb};
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
    let host: String;
    let default_remote_dir: String;
            
    if args.hostname{
        
        let ip = args.ip.unwrap();
        host = format!("hostname@{}", ip);
        default_remote_dir = "/home/hostname/public_html".to_string();
    
    }else{

        let node_id = args.node_id.unwrap();
        let node = Node::new(&node_id, &pool);
        let node_data = node.data().await?;
        host = node.hostname().await?;
        let ssh_username = node_data.ssh.username;
        
        if node_data.home.is_some(){
            default_remote_dir = node_data.home.unwrap();
        }else{
            default_remote_dir = format!("/home/{}", ssh_username);
        }


        if node.ssh().await.is_ok() {
            shell_exec(&format!("webman push pbk {node_id}"))
        }

    }

    shell_exec(&format!("code --remote ssh-remote+{} {}", host, default_remote_dir));

    Ok(())
}