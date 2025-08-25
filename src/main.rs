pub mod libs;
use crate::libs::hello::sayhello;
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "Youre")]
    name: String,
}

fn main() {
    let args = Args::parse();
    let name = sayhello(args.name);
    println!("{}, new world!!", name);
}
