use serde_derive::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use crate::{BUCKET_REFRESH_INTERVAL, K};
use crate::node::NodeInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct KadBucket {
    nodes: Vec<NodeInfo>,
    last_update: OffsetDateTime
}

impl KadBucket {
    fn new() -> Self {
        KadBucket {
            nodes: Vec::new(),
            last_update: OffsetDateTime::now_utc()
        }
    }

    pub fn nodes(&self) -> &[NodeInfo] {
        self.nodes.as_slice()
    }

    pub fn update_node(&mut self, node_data: NodeInfo) {
        self.last_update = OffsetDateTime::now_utc();
        if let Some(index) = self.nodes.iter().position(|d| *d == node_data) {
            self.nodes.remove(index);
        }
        self.nodes.push(node_data);
        if self.nodes.len() > K as usize {
            self.nodes.remove(0);
        }
    }

    pub fn is_stale(&self) -> bool {
        let diff = OffsetDateTime::now_utc() - self.last_update;
        diff > Duration::seconds(BUCKET_REFRESH_INTERVAL)
    }
    
    pub fn delete_node(&mut self, node: &NodeInfo) {
        if let Some(idx) = self.nodes.iter().position(|data| data == node) {
            self.nodes.remove(idx);
        }
    }

    pub fn summary(&self) -> String {
        format!("{} nodes, last update {}", self.nodes.len(), self.last_update)
    }
}

impl Default for KadBucket {
    fn default() -> Self {
        KadBucket::new()
    }
}