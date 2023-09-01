use std::{io::IsTerminal, process::ExitCode};
use std::error::Error;

use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use kaphene::cluster::Cluster;
use kaphene::instrumentation;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<ExitCode, dyn Error>{
    HookBuilder::default()
        .theme(if !std::io::stderr().is_terminal(){
            Theme::new()
        } else {
            Theme::dark()
        })
        .install()?;

    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Start(args)) => {
            let cluster = Cluster::from_config(args.config.unwrap_or_else("config.toml"));
            cluster.start().await?;
        },
        _ => {
            println!("command not found");
            std::process::exit(1);
        }
    }

    Ok(ExitCode::SUCCESS)
}

#[derive(Parser)]
pub struct Cli {
    #[clap(flatten)]
    pub args: Args,
    #[clap(flatten)]
    pub instrumentation: instrumentation::Instrumentation,
}

#[derive(Parser)]
#[derive(Debug)]
pub struct Args {
    config: Option<String>,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Commands {
    Start(Args),
}