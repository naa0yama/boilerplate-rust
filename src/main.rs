pub mod libs;
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
    use tracing_subscriber::{filter::EnvFilter, fmt};
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let args = Args::parse();
    if args.version {
        tracing::info!("{}", APP_VERSION);
        std::process::exit(0);
    }

    run(args.name);
}

pub fn run(name: String) {
    use crate::libs::hello::sayhello;
    let greeting = sayhello(name);
    tracing::info!("{}, new world!!", greeting);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::subscriber::with_default;
    use tracing_mock::{expect, subscriber};

    #[test]
    fn test_run_with_default_name() {
        let (subscriber, handle) = subscriber::mock()
            .event(expect::event().with_fields(expect::msg("Hi, Youre, new world!!")))
            .only()
            .run_with_handle();

        with_default(subscriber, || {
            run("Youre".to_string());
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_custom_name() {
        let (subscriber, handle) = subscriber::mock()
            .event(expect::event().with_fields(expect::msg("Hi, Alice, new world!!")))
            .only()
            .run_with_handle();

        with_default(subscriber, || {
            run("Alice".to_string());
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_empty_name() {
        let (subscriber, handle) = subscriber::mock()
            .event(expect::event().with_fields(expect::msg("Hi, , new world!!")))
            .only()
            .run_with_handle();

        with_default(subscriber, || {
            run("".to_string());
        });

        handle.assert_finished();
    }

    #[test]
    fn test_run_with_japanese_name() {
        let (subscriber, handle) = subscriber::mock()
            .event(expect::event().with_fields(expect::msg("Hi, 世界, new world!!")))
            .only()
            .run_with_handle();

        with_default(subscriber, || {
            run("世界".to_string());
        });

        handle.assert_finished();
    }
}
