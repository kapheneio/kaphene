use std::fmt::Formatter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("lock error")]
    Lock,
    #[error("node error")]
    Node,
    #[error("server error")]
    Server,
    #[error("socket error")]
    Socket,
}

pub type Result<T, E = Report> = color_eyre::Result<T, E>;

// Matches:
// `Err(some_err).wrap_err("Some context")`
// `Err(color_eyre::eyre::Report::new(SomeError))`
pub struct Report(color_eyre::Report);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where E: Into<color_eyre::Report>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}