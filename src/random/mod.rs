mod digits;
mod characters;
mod hex;


#[derive(clap::Args)]
#[command(arg_required_else_help(true))]
pub struct Args{
    #[command(subcommand)]
    pub command: Option<Commands>
}


#[derive(clap::Subcommand)]
pub enum Commands{

    Digits(digits::Args),

    Characters(characters::Args),

    Hex(hex::Args)

}


pub fn action(cmd: Commands){

    match cmd {
        
        Commands::Digits(args) => digits::action(args),

        Commands::Characters(args) => characters::action(args),

        Commands::Hex(args) => hex::action(args)

    }
}