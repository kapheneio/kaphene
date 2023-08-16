use clap::Parser;

use crate::Error;
use crate::instrumentation;
use crate::logger;

pub const DEFAULT_SOCKET_PATH: &'static str = "/tmp/kaphene.sock";

pub enum AgentKind {
    Node,
    Server,
}

pub struct AgentOpts {
    pub socket_path: Option<String>,
    pub kind: AgentKind,
}

pub trait Agent {
    async fn new(mut opts: AgentOpts) -> Result<(), Error> {
        if opts.socket_path.is_none() {
            opts.socket_path = Some(DEFAULT_SOCKET_PATH.to_string());
        }

        Agent::start(opts)
    }

    async fn start(opts: AgentOpts) -> Result<(), Error>;
}

#[derive(Parser)]
pub(crate) struct Cli {
    #[clap(flatten)]
    pub(crate) instrumentation: instrumentation::Instrumentation,
}