use clap::{Parser, Subcommand};
use eyre::Result;

mod node;
mod nodes;
mod servers;
mod db;
mod devserver;
mod code;
mod ssh;
mod remote;
mod push;
mod config;
mod ssh_config;
mod beta;
mod random;



#[derive(Parser)]
#[command(name = "webman", version, about = "Manage major local activities", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}



#[derive(Subcommand)]
enum Commands {

    #[command(about = "Manage a node")]
    Node(node::Args),

    #[command(about = "Manage nodes")]
    Nodes(nodes::Args),

    #[command(about = "Manage servers")]
    Servers(servers::Args),

    #[command(about = "Manage SQLite database and generate models")]
    Db(db::Args),

    #[command(about = "Output the devserver")]
    Devserver(devserver::Args),

    #[command(about = "SSH into node")]
    Ssh(ssh::Args),

    #[command(about = "Open node in VSCode remote SSH")]
    Code(code::Args),

    #[command(about = "VS Code from from ssh params")]
    Remote(remote::Args),

    #[command(about = "Push objects to remote locations")]
    Push(push::Args),

    #[command(about = "Config basic things")]
    Config(config::Args),

    #[command(about = "Activities on ssh-config")]
    SshConfig(ssh_config::Args),

    #[command(about = "Access and activities on beta / developmental node")]
    Beta(beta::Args),

    #[command(about = "Generate random characters")]
    Random(random::Args)
}


#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();

    match cli.command{

        Commands::Node(args) => node::action(args).await?,

        Commands::Nodes(args) => nodes::action(args).await?,

        Commands::Servers(args) => servers::action(args).await?,

        Commands::Db(args) => db::action(args).await?,

        Commands::Devserver(args) => devserver::action(args).await?,

        Commands::Ssh(args) => ssh::action(args).await?,

        Commands::Code(args) => code::action(args).await?,

        Commands::Remote(args) => remote::action(args),

        Commands::Push(args) => push::action(args.command.unwrap()).await?,

        Commands::Config(args) => config::action(args).await?,

        Commands::SshConfig(args) => ssh_config::action(args.command.unwrap()).await?,

        Commands::Beta(args) => beta::action(args.command.unwrap()).await?,

        Commands::Random(args) => random::action(args.command.unwrap())

    }

    Ok(())

}