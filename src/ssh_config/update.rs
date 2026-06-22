use prelude::debug_mode;
use ssh_cfg::{SshConfigParser, SshOptionKey};
use util::{file_get_contents, file_put_contents, home_dir};


#[derive(clap::Args)]
pub struct Args{

    host: String,

    #[arg(long)]
    hostname: Option<String>

}


pub async fn action(args: Args){

    let config_path = format!("{home}/.ssh/config", home = home_dir());

    let config_string = file_get_contents(&config_path);

    let config_string_with_includes = config_string
        .lines()
        .filter(|line| line.starts_with("Include"))
        .collect::<Vec<_>>()
        .join("\n");

    let config_string_without_includes = config_string
        .lines()
        .filter(|line| !line.starts_with("Include"))
        .collect::<Vec<_>>()
        .join("\n");

    if debug_mode(){
        println!("config_string_with_includes");
        println!("{config_string_with_includes}");
        println!();
        println!();
        println!("config_string_without_includes");
        println!("{config_string_without_includes}");
    }
 
    let mut ssh_config = SshConfigParser::parse_config_contents(&config_string_without_includes).unwrap();

    let host = args.host;

    if ssh_config.contains_key(&host){
        
        let host_config = ssh_config.get_mut(&host).unwrap();

        if let Some(hostname) = args.hostname{
            host_config.insert(SshOptionKey::HostName, hostname);
        }

        let mut new_config_string = config_string_with_includes;

        new_config_string = format!("{new_config_string}\n");

        for (host, host_config) in ssh_config.iter(){
            
            new_config_string = format!("{new_config_string}\n\n\nHost {host}");
            
            for (key, value) in host_config.iter(){
                new_config_string = format!("{new_config_string}\n\t{key} {value}");
            }

        }

        new_config_string = format!("{new_config_string}\n\n\n");

        file_put_contents(&config_path, &new_config_string);

    }else{
        println!("{host} not found in ssh config");
    }

}