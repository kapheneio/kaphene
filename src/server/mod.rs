mod task;

use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use async_trait::async_trait;
use bytes::BytesMut;
use eyre::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::net::unix::SocketAddr;
use tokio::sync::{mpsc, Notify};
use crate::agent::{Agent, Opts};
use crate::{Error, Result};
use crate::error::Report;
use crate::server::task::{Task, TaskHandle};

pub struct Server {
    listener: UnixListener,
    socket_path: String,
    opts: Opts,
    tasks: HashMap<TaskHandle>,
}

#[async_trait]
impl Agent for Server {
    async fn start<'a>(opts: Opts, shutdown: impl Future + Send + Debug) -> Result<()> {
        let path = Agent::init_socket(opts.socket_path.expect("invalid socket path").as_str()).await;

        let server = UnixListener::bind(path)
            .map(|listener| Server { listener, socket_path: path.to_string(), opts })
            .wrap_err(Error::Socket)
            .expect("server bind failure");

        // Keep an active task count so that we can attempt a graceful shutdown.
        let active_task_count = Arc::new((AtomicUsize::new(0), Notify::new()));
        // Establish channels for coordinating task counts.
        let (tx, rx) = mpsc::channel(1_024);

        loop {
            tokio::select! {
                result = server.accept() => {
                    let (socket, addr) = result.unwrap();
                    let active_task_count = active_task_count.clone();

                    active_task_count.0.fetch_add(1, Ordering::Relaxed);

                    tokio::spawn(async move {
                        if let Err(e) = server.handle(socket, addr).await {
                            if let Err(_) = tx.send(_).await {
                                println!("task count receiver dropped");
                                return;
                            }

                            return Err(e);
                        }

                        if let Err(_) = tx.send(_).await {
                            println!("task count receiver dropped");
                            return;
                        }
                    });

                    rx.for_each(|_| {
                        // Decrement task count.
                        let count = active_task_count.0.fetch_sub(1, Ordering::Relaxed);
                        // If last task, notify complete.
                        if count == 1 {
                            active_task_count.1.notify_one();
                        }
                    })
                }
                _ = shutdown => {
                    println!("==> shut down received");

                    // Wait 30 sec or until all tasks complete, whichever comes first.
                    let timer = tokio::time::sleep(Duration::from_secs(30));
                    let tasks_complete = active_task_count.1.notified();

                    // Block until one condition occurs.
                    if active_task_count.0.load(Ordering::Relaxed) != 0 {
                        select!{
                            _ = timer => println!("==> shutting down with pending tasks"),
                            _ = tasks_complete => {}
                        }
                    }

                    println!("==> shut down complete");
                    drop(server);
                }
            }
        }
    }
}

impl Server {
    pub async fn handle(&self, mut socket: UnixStream, addr: SocketAddr) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(1024);
        self.socket.read_buf(&mut buffer).await?;

        if buffer.is_empty() {
            eprintln!("==> empty buffer");
            return Err(Report::from(Error::Socket("empty buffer".to_string())));
        }

        let task = String::from_utf8_lossy(buffer.as_ref());

        if task.is_empty() {
            eprintln!("==> empty task");
            return Err(Report::from(Error::Socket("empty task".to_string())));
        }

        println!("==> processing incoming command from: {:?}", addr);

        let task: Task = match serde_json::from_str(task.as_ref()) {
            Ok(t) => t,
            Err(e) => {
                return Err(Report::from(Error::Server(format!("bad task: {e}"))));
            }
        };

        let result = match self.dispatch(task).await {
            Ok(r) => match r {
                None => "task dispatched".to_string(),
                Some(r) => r,
            },
            Err(e) => format!("error dispatching tasks: {}", e),
        };

        match socket.write_all(result.as_bytes()).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("==> error writing to socket: {}", e);
                return Err(Report::from(Error::Socket(e.to_string())));
            }
        }

        Ok(())
    }

    async fn dispatch(&self, task: &Task) -> Result<String> {

    }
}

impl Drop for Server {
    fn drop(&mut self) {
        std::fs::remove_file(&self.socket_path).unwrap();
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
