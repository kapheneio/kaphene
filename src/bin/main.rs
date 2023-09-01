use std::{io::IsTerminal, process::ExitCode};
use std::error::Error;


use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use kaphene::agent::{Agent, Cli};
use tokio::signal;

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
    cli.instrumentation.init()?;

    Agent::new(cli.opts, signal::ctrl_c()).await?;

    Ok(ExitCode::SUCCESS)
}