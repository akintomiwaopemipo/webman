

#[derive(clap::Args)]
pub struct Args{
    hostname: String,

    home: Option<String>,
}



pub fn action(args: Args){
        
    let hostname = args.hostname;
    let home = args.home;

    util::shell_exec(
        &format!("code --remote ssh-remote+{} {}", hostname, home.unwrap_or(String::from("")))
    );
}