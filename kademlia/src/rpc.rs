use serde_derive::{Deserialize, Serialize};
use crate::{KadError, KadID, Result};
use crate::node::NodeInfo;

#[derive(Serialize,Deserialize)]
enum KadMessage {
    Request(KadRequest),
    Response(KadResponse),
}

impl KadMessage {
    fn as_message(&self) -> Result<String> {
        serde_json::to_string(self).map_err(KadError::MessageSerializeError)
    }

    fn from_string(data: &str) -> Result<Self> {
        serde_json::from_str(data).map_err(KadError::MessageDeserializeError)
    }
}

#[derive(Serialize,Deserialize)]
struct KadRequest {
    uid: [u8; 20],
    function: KadRequestFunction,
}

#[derive(Serialize,Deserialize)]
enum KadRequestFunction {
    Ping,
    Store { key: KadID, value: Vec<u8> },
    FindNode { id: KadID },
    FindValue { key: KadID },
}

#[derive(Serialize,Deserialize)]
struct KadResponse {
    uid: [u8; 20],
    function: KadResponseFunction,
}

#[derive(Serialize,Deserialize)]
enum KadResponseFunction {
    Ping,
    Store, // TODO: What here
    FindNode { nodes: Vec<NodeInfo> },
    FindValue(KadFindValueResponse)
}

#[derive(Serialize,Deserialize)]
enum KadFindValueResponse {
    Found(Vec<u8>),
    Next(Vec<NodeInfo>)
}