use app::{config::{Node, Server}, config_db::ConfigDb};
use tokio::fs;
use util::default_ppk_path;
use colored::Colorize;
use utils::ssh::Session;
use eyre::Result;


#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "The node ID of the node, also the hostname of the SSH config or ssh address on server")]
    context: Option<String>,

    #[arg(long, help = "IP Address of the root node")]
    ip: Option<String>,

    #[arg(short = 'z', long, help = "IP Address of root in config")]
    root_ip: Option<String>
}



pub async fn action(args: Args) -> Result<()> {

    let pool = ConfigDb::connection_pool().await?;

    if let Some(context) = args.context{

        if context.contains("@"){
            
            let mut ssh_address = context.split("@");
            let username = ssh_address.next().unwrap();
            let host = ssh_address.next().unwrap();

            let password = rpassword::prompt_password("Password: ".bright_cyan()).unwrap();

            let mut ssh = Session::connect(host, username, &password, None).await?;


            let ppk_path = default_ppk_path();
    
            let pbk_path = format!("{ppk_path}.pub");

            let remote_home_path =  if username == "root" {
                format!("/root")
            } else {
                format!("/home/{username}")
            };

            let remote_path = format!("{remote_home_path}/authorized_keys.chunk");

            ssh.upload(&remote_path, &fs::read(&pbk_path).await?).await?;

            ssh.exec("mkdir -p .ssh && chmod 700 .ssh && cat authorized_keys.chunk >> .ssh/authorized_keys && chmod 644 .ssh/authorized_keys && rm authorized_keys.chunk").await?;

            println!("Successfully uploaded public key")


        }else{
            let node_id = context;
            let node = Node::new(&node_id, &pool);
            match node.ssh().await {
                Ok(mut ssh) => {
                    
                    let node = node.data().await?;
                    
                    let ppk_path = node.ssh.private_key.unwrap_or(util::default_ppk_path());
    
                    let pbk_path = format!("{}.pub", ppk_path);
    
                    let remote_path = format!("{}/authorized_keys.chunk", node.remote_home_dir.unwrap_or(format!("/home/{}", node.ssh.username)));
    
                    ssh.upload(&remote_path, &fs::read(&pbk_path).await?).await?;
    
                    ssh.exec("mkdir -p .ssh && chmod 700 .ssh && cat authorized_keys.chunk >> .ssh/authorized_keys && chmod 644 .ssh/authorized_keys && rm authorized_keys.chunk").await?;
    
                    println!("Successfully uploaded public key")
    
                },
                Err(_) => {
                    println!("Could to connect to host");
                    std::process::exit(1);
                }
            }
        }
        
    }else if let Some(root_ip) = args.root_ip {
        let server = Server::new(&root_ip, &pool);
        match server.ssh().await {
            Ok(mut ssh) => {

                let pbk_path = &format!("{}.pub", util::default_ppk_path());

                let remote_path = format!("{}/authorized_keys.chunk","/root");

                ssh.upload(&remote_path, &fs::read(&pbk_path).await?).await?;

                ssh.exec("mkdir -p .ssh && chmod 700 .ssh && cat authorized_keys.chunk >> .ssh/authorized_keys && chmod 644 .ssh/authorized_keys && rm authorized_keys.chunk").await?;

                println!("Successfully uploaded public key")

            },
            Err(_) => {

            }
        }    

    }

    Ok(())
    
}