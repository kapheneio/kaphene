use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Message {
    Bpf{ fn_name: String },
    HealthCheck,
    Workload { name: String, dataset: String }
}