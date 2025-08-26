pub mod libs;
use crate::libs::hello::sayhello;
use clap::Parser;

#[derive(Parser)]
#[command(about)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "Youre")]
    name: String,
    #[arg(long, short = 'V', help = "Print version")]
    version: bool,
}

const APP_VERSION: &str = concat!(
    env!("CARGO_PKG_NAME"),
    " version ",
    env!("CARGO_PKG_VERSION"),
    " (rev:",
    env!("GIT_HASH"),
    ")\n",
);

fn main() {
    let args = Args::parse();
    if args.version {
        print!("{}", APP_VERSION);
        std::process::exit(0);
    }

    run(args.name);
}

pub fn run(name: String) {
    let greeting = sayhello(name);
    println!("{}, new world!!", greeting);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_default_name() {
        run("Youre".to_string());
    }

    #[test]
    fn test_run_with_custom_name() {
        run("Alice".to_string());
    }

    #[test]
    fn test_run_with_empty_name() {
        run("".to_string());
    }

    #[test]
    fn test_run_with_japanese_name() {
        run("世界".to_string());
    }
}
