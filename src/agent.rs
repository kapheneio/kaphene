use std::fmt::Debug;
use std::future::Future;
use std::path::Path;

use async_trait;
use clap::Parser;
use eyre::Context;
use tokio::net::unix::SocketAddr;

use crate::{Error, Result};
use crate::instrumentation;
use crate::node::Node;
use crate::server::Server;

pub const DEFAULT_NODE_SOCKET_PATH: &'static str = "/tmp/kaphd-node.sock";
pub const DEFAULT_SERVER_SOCKET_PATH: &'static str = "/tmp/kaphd-server.sock";

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Kind {
    Node,
    Server,
}

#[derive(Parser)]
#[derive(Debug)]
pub struct Opts {
    pub name: String,
    pub kind: Kind,
    pub socket_path: Option<String>,
    pub join: Option<SocketAddr>,
    pub nodes: Option<Vec<Opts>>,
}

#[async_trait::async_trait]
pub trait Agent {

    async fn new(mut opts: Opts, shutdown: impl Future + Send + Debug) -> Result<()> {
        if opts.socket_path.is_none() {
            opts.socket_path = Some(DEFAULT_SERVER_SOCKET_PATH.to_string());
        }

        match opts.kind {
            Kind::Node => Node::start(opts, shutdown).await,
            Kind::Server => Server::start(opts, shutdown).await,
        }
    }

    async fn init_socket(path: &str) -> Result<&str> {
        let socket_path = Path::new(path);

        // Naively believes that if the socket file exists the agent is running.
        if socket_path.exists() {
            println!("==> socket exists: shutting down existing instance before attempting to start a new one.");
            tokio::fs::remove_file(path.clone())
                .await
                .wrap_err(Error::Socket)
                .expect("cannot remove previous socket file");
        }

        Ok(path)
    }

    async fn start<'a>(opts: Opts, shutdown: impl Future + Send + Debug) -> Result<()>;
}

#[derive(Parser)]
pub struct Cli {
    #[clap(flatten)]
    pub opts: Opts,
    #[clap(flatten)]
    pub instrumentation: instrumentation::Instrumentation,
}