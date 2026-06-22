use std::process::exit;

use app::config::{Config, Server};
use indexmap::IndexMap;
use prelude::{SerdeJsonSerialize, SerdeJsonValueSerialize};
use serde_json::json;
use util::shell_exec;
use eyre::Result;

#[derive(clap::Args)]
pub struct Args{

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

    let roots = Server::list(&pool).await?;

    let filtered_roots = roots
        .into_iter()
        .filter(|root| root.1.hostname.contains("devserver"))
        .collect::<IndexMap<_,_>>();

    let devserver = filtered_roots.first();

    if let Some(devserver) = devserver{

        let root_ip = devserver.0;
        let server = Server::new(root_ip, &pool);

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
            let mut root_value = json!({ "ip": devserver.0 });
            root_value.merge(&devserver.1.clone().json_value());
            root_value.pretty_print();
        } else {
            println!("{root_ip}");
        }   
    }

    Ok(())
}