use std::future::Future;

use clap::Parser;

use crate::Error;
use crate::instrumentation;
use crate::node::Node;
use crate::server::Server;

pub const DEFAULT_SOCKET_PATH: &'static str = "/tmp/kaphd.sock";

pub enum AgentKind {
    Node,
    Server,
}

pub struct AgentOpts {
    pub socket_path: Option<String>,
    pub kind: AgentKind,
}

pub trait Agent {

    async fn new(mut opts: AgentOpts, shutdown: impl Future) -> Result<(), Error> {
        if opts.socket_path.is_none() {
            opts.socket_path = Some(DEFAULT_SOCKET_PATH.to_string());
        }

        match opts.kind {
            AgentKind::Node => Node::start(opts, shutdown),
            AgentKind::Server => Server::start(opts, shutdown),
        }
    }

    #[tracing::instrument]
    async fn start(opts: AgentOpts, shutdown: impl Future) -> Result<(), Error>;
}

#[derive(Parser)]
pub struct Cli {
    #[clap(flatten)]
    pub opts: AgentOpts,
    #[clap(flatten)]
    pub instrumentation: instrumentation::Instrumentation,
}