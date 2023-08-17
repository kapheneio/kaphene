use color_eyre::config::{HookBuilder, Theme};
use kaphene::Result;
use kaphene::agent::{Agent, Cli};

use clap::Parser;
use std::{io::IsTerminal, process::ExitCode};
use tokio::signal;

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<ExitCode>{
    HookBuilder::default()
        .theme(if !std::io::stderr().is_terminal(){
            Theme::new()
        } else {
            Theme::dark()
        })
        .install()?;


    let cli = Cli::parse();
    cli.instrumentation.init()?;

    Agent::new(cli.opts, signal::ctrl_c()).await?;

    Ok(ExitCode::SUCCESS)
}