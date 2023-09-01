use serde::{Deserialize, Serialize};
use tokio::net::unix::SocketAddr;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Task {
    kind: Kind,
    message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum Kind {
    Bpf,
    HealthCheck,
    Workload
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TaskHandle {
    pub id: String,
    pub task: Task,
    pub node_addr: SocketAddr,
    sender: Sender<Task>
}

impl TaskHandle {
    pub(crate) async fn
}
