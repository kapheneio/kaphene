use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("lock error: {0}")]
    Lock(String),
    #[error("node error: {0}")]
    Node(String),
    #[error("server error: {0}")]
    Server(String),
    #[error("socket error: {0}")]
    Socket(String),
}