use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use eyre::Context;
use tokio::net::{UnixListener, UnixStream};
use tokio::net::unix::SocketAddr;
use crate::agent::{Agent, AgentOpts};
use crate::{Error, Result};

pub struct Server {
    listener: UnixListener,
    socket_path: String,
}

impl Agent for Server {
    async fn start(opts: AgentOpts, shutdown: impl Future) -> Result<()> {
        let path = opts.socket_path.expect("invalid socket path").as_str();
        let socket_path = Path::new(path);

        // Naively believes that if the socket file exists the agent is running.
        if socket_path.exists() {
            println!("==> socket exists: shutting down existing instance before attempting to start a new one.");
            tokio::fs::remove_file(path)
                .await
                .wrap_err(Error::Socket)
                .expect("cannot remove previous socket file");
        }

        let server = UnixListener::bind(opts.path)
            .map(|listener| Server { listener, socket_path: path.to_string() })
            .wrap_err(Error::Socket)
            .expect("server bind failure");

        tokio::select! {
            response = server.run() => {
                if let Err(e) = response {
                    return Err(e).wrap_err(Error::Server)
                }
            }
            _ = shutdown => {
                println!("kaphd server shut down received");
                drop(server);
            }
        }

        Ok(())
    }
}

impl Server {
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
                match dispatcher.send().await {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("error dispatching message: {}", e);
                    }
                }
            });
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        tokio::fs::remove_file(&self.socket_path).unwrap();
    }
}

impl Deref for Server {
    type Target = UnixListener;

    fn deref(&self) -> &Self::Target {
        &self.listener
    }
}

impl DerefMut for Server {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.listener
    }
}

pub(crate) struct Dispatcher {
    socket: UnixStream,
    addr: SocketAddr,
}

impl Dispatcher {
    pub(crate) async fn send(&self) -> Result<()> {

    }
}