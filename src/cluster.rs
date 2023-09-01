use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use tokio::signal;

use crate::{Error, Result};
use crate::agent::{Agent, DEFAULT_NODE_SOCKET_PATH, DEFAULT_SERVER_SOCKET_PATH, Kind, Opts};
use crate::error::Report;
use crate::server::Server;

pub struct Cluster {
    pub leader: Opts,
    pub servers: HashMap<String, Opts>,
    pub nodes: HashMap<String, Opts>,
}

/// Cluster configuration loaded from config.toml.
pub(crate) struct Config {
    pub(crate) servers: Option<Vec<Opts>>,
    pub(crate) nodes: Option<Vec<Opts>>,
}

impl Cluster {
    pub fn from_config(config_path: String) -> Result<Cluster> {
        let mut file = File::open(config_path).expect("failed to open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("failed to read config file");

        let config: Config = toml::from_str(contents.as_str()).expect("failed to deserialize config");

        if config.servers.len() > 1 {
            return Err(Report::from(Error::Config("only single server mode is supported at this time".to_string())));
        }

        // Create default leader
        let mut leader = Opts{
            name: "server-1".to_string(),
            kind: Kind::Server,
            socket_path: Some(DEFAULT_SERVER_SOCKET_PATH.to_string()),
            join: None,
        };
        let servers = match config.servers {
            None => {
                HashMap::from([("server-1".to_string(), leader.clone())])
            },
            Some(mut s) => {
                let opts = s.get_mut(0).expect("invalid server options");
                let mut opts = opts.clone();
                let mut servers = HashMap::new();
                if opts.socket_path.is_none() {
                    opts.socket_path = format!("/tmp/kaphd-{}.sock", opts.name);
                };
                servers.push(opts.name.clone(), opts);
                servers
            }
        };

        let nodes = match config.nodes {
            None => HashMap::from([("node-1".to_string(), Opts{
                name: "node-1".to_string(),
                kind: Kind::Node,
                socket_path: Some(DEFAULT_NODE_SOCKET_PATH.to_string()),
                join: None,
            })]),
            Some(mut n) => {
                let opts = n.get_mut(0).expect("invalid node options");
                let mut opts = opts.clone();
                let mut nodes = HashMap::new();
                if opts.socket_path.is_none() {
                    opts.socket_path = format!("/tmp/kaphd-{}.sock", opts.name);
                };
                nodes.push(opts.name.clone(), opts);
                nodes
            }
        };

        Ok(Cluster{ leader, servers, nodes })
    }

    pub(crate) async fn start(&self) -> Result<()> {
        Server::start(self.leader.clone(), signal::ctrl_c()).await?;
        Ok(())
    }
}