
#[derive(clap::Args)]
pub struct Args{
    #[arg(help = "Length of characters")]
    length: usize,
}


pub fn action(args: Args){
    println!("{}", util::random_digits(args.length))
}