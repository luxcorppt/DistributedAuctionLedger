use serde_derive::{Deserialize, Serialize};
use crate::{KadError, MESSAGE_BUFFER_LENGTH};
use crate::kadid::KadID;
use crate::node::NodeInfo;

#[derive(Serialize,Deserialize,Debug)]
pub(crate) struct KadMessage {
    // TODO: Restrict access to fields
    pub(crate) sender: NodeInfo,
    pub(crate) payload: KadMessagePayload
}

impl KadMessage {
    pub(crate) fn as_message(&self) -> Result<String, KadError> {
        let res = serde_json::to_string(self).map_err(KadError::MessageSerializeError)?;
        assert!(res.len() <= MESSAGE_BUFFER_LENGTH);
        Ok(res)
    }

    pub(crate) fn from_string(data: &str) -> Result<Self, KadError> {
        serde_json::from_str(data).map_err(KadError::MessageDeserializeError)
    }
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) enum KadMessagePayload {
    Request(KadRequest),
    Response(KadResponse),
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) struct KadRequest {
    pub(crate) uid: [u8; 20],
    pub(crate) function: KadRequestFunction,
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) enum KadRequestFunction {
    Ping,
    Store { key: KadID, value: Vec<u8> },
    FindNode { id: KadID },
    FindValue { key: KadID },
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) struct KadResponse {
    pub(crate) uid: [u8; 20],
    pub(crate) function: KadResponseFunction,
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) enum KadResponseFunction {
    Ping,
    FindNode { nodes: Vec<NodeInfo> },
    FindValue(KadFindValueResponse)
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) enum KadFindValueResponse {
    Found(Vec<u8>),
    Next(Vec<NodeInfo>)
}
