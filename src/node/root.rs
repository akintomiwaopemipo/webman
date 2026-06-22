use std::process::exit;
use eyre::Result;
use app::config::{Config, Node, Server};
use indexmap::indexmap;
use prelude::SerdeJsonSerialize;
use util::shell_exec;



#[derive(clap::Args)]
pub struct Args{
    node_id: String,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    ssh: bool,

    #[arg(long)]
    code: bool,

    #[arg(long)]
    command: bool,

    #[arg(short, long)]
    dir: Option<String>
}


pub async fn action(args: Args) -> Result<()> {
    let pool = Config::connection_pool().await?;
    let node_id = args.node_id;
    let node = Node::new(&node_id, &pool);
    let node_data = node.data().await?;
    let root_ip = node_data.host;

    let server = Server::new(&root_ip, &pool);
    let server_data = server.data().await?;

    let dir = args.dir.unwrap_or("/".to_string());

    if args.code || (args.ssh && !args.command){
        if !server.ssh().await.is_ok() { shell_exec(&format!("webman push pbk --root-ip {root_ip}")) }
    }


    if args.code{
        shell_exec(&format!("code --remote ssh-remote+root@{root_ip} {dir}"));
        exit(0);
    }

    if args.ssh{
        let command_prefix = format!(r#"ssh -o ServerAliveInterval=300 root@{root_ip}"#);
        if args.command{
            println!("{command_prefix}");
        }else{
            shell_exec(&format!(r#"{command_prefix} -t "cd {dir} ; bash --login" "#));
        }

        exit(0);
    }

    
    if args.json{
        let json = indexmap! { root_ip => server_data };
        println!("{}", json.stringify_pretty());
        exit(0);
    } 
    
    println!("Root IP: {root_ip}");

    Ok(())

}