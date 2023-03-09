use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use crate::{KadError, KadID};

#[derive(Serialize,Deserialize)]
pub struct NodeInfo {
    socket_addr: SocketAddr,
    id: KadID,
}

#[derive(Serialize,Deserialize)]
struct LocalNode<Status> {
    #[serde(skip)]
    status: Status,
    local_id: KadID,
    buckets: Vec<VecDeque<NodeInfo>>, // Vec aqui é ineficiente, mas é simples de implementar
    storage: HashMap<KadID, Vec<u8>>,
    // OPTIONAL: Implement Lookup caching
}

#[derive(Default)]
struct Init;

#[derive(Default)]
struct Ready;

impl LocalNode<Init> {
    fn start_empty() -> Self {
        let id = KadID::random();
        let mut b: Vec<_> = Default::default();
        b.resize_with(160, Default::default);
        let s = Default::default();

        LocalNode {
            local_id:id,
            buckets:b,
            storage:s,
            status: Init
        }
    }

    fn start_from_data(data: &str) -> crate::Result<Self> {
        serde_json::from_str(data).map_err(KadError::LocalDeserializeError)
    }

    fn prepare(self) -> LocalNode<Ready> {
        todo!()
    }
}
