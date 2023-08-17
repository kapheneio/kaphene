use std::future::Future;
use tokio::net::UnixListener;
use crate::agent::{Agent, AgentOpts};
use crate::Error;

pub struct Node {
    listener: UnixListener,
}

impl Agent for Node {
    async fn start(opts: AgentOpts, shutdown: impl Future) -> Result<(), Error> {
        todo!()
    }
}