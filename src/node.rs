use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use async_trait::async_trait;
use bytes::BytesMut;
use eyre::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::net::unix::SocketAddr;

use crate::agent::{Agent, Opts};
use crate::{Error, Result};
use crate::error::Report;
use crate::message::Message;

pub struct Node {
    listener: UnixListener,
    socket_path: String,
    join: SocketAddr,
}

#[async_trait]
impl Agent for Node {
    async fn start<'a>(opts: Opts, shutdown: impl Future + Send) -> Result<()> {
        let join = opts.join.expect("nodes must join a server");
        let path = opts.socket_path.expect("invalid socket path").as_str();
        let socket_path = Path::new(path);

        // Naively believes that if the socket file exists the agent is running.
        if socket_path.exists() {
            println!("==> socket exists: shutting down existing instance before attempting to start a new one.");
            tokio::fs::remove_file(path.clone())
                .await
                .wrap_err(Error::Socket("cannot remove previous socket file".to_string()))?;
        }

        let node = UnixListener::bind(path)
            .map(|listener| Node { listener, socket_path: path.to_string(), join })
            .wrap_err(Error::Socket("node bind failure".to_string()))?;

        tokio::select! {
            response = node.run() => {
                if let Err(e) = response {
                    return Err(crate::error::Report(Error::Node.into()));
                }
            }
            _ = shutdown => {
                println!("kaphd node shut down received");
                drop(node);
            }
        }

        Ok(())
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        std::fs::remove_file(&self.socket_path).unwrap();
    }
}

impl Deref for Node {
    type Target = UnixListener;

    fn deref(&self) -> &Self::Target {
        &self.listener
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.listener
    }
}

impl Node {
    pub async fn run(&self) -> Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await
                .wrap_err(Error::Server)
                .expect("listener accept failure");


            let mut dispatcher = Dispatcher {
                socket,
                addr,
            };

            tokio::spawn(async move {
                match dispatcher.dispatch().await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("error routing command: {}", e);
                    }
                }
            });
        }
    }
}

pub struct Dispatcher {
    socket: UnixStream,
    addr: SocketAddr,
}

impl Dispatcher {
    pub(crate) async fn dispatch(&mut self) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(1024);
        self.socket.read_buf(&mut buffer).await?;

        if buffer.is_empty() {
            eprintln!("==> empty message buffer");
            return Err(Report::from(Error::Socket("empty message buffer".to_string())));
        }

        let message = String::from_utf8_lossy(buffer.as_ref());

        if message.is_empty() {
            eprintln!("==> empty command");
            return Err(Report::from(Error::Socket("empty message".to_string())));
        }

        println!(
            "==> incoming message from: {:?}",
            self.addr
        );
        
        let message: &dyn Message<Result=()> = serde_json::from_str(message.as_str())?;

        let result = match self.session.dispatch(message.as_ref()) {
            Ok(r) => match r {
                None => "command executed successfully".to_string(),
                Some(r) => r,
            },
            Err(e) => format!("error executing command: {}", e),
        };

        match self.socket.write_all(result.as_bytes()).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("==> error writing to socket: {}", e);
                return Err(Report::from(Error::Socket(e.to_string())));
            }
        }
        Ok(())

    }
}

pub trait Actor<T>
where T: Message {
    type Target: T;

    fn invoke(&self) -> Result<dyn Message<Result=String>>;
}

pub trait ActorFactory {
    fn resolve<T>(&self, message: T) -> Box<dyn Actor<T, Target=T>>;
}


