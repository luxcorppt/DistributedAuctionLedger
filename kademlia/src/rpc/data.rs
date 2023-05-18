use ed25519_dalek_fiat::{Keypair, Signature, Signer, Verifier};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use crate::{KadError, MESSAGE_BUFFER_LENGTH, SECURE_KEY_C2, util};
use crate::kadid::KadID;
use crate::node::NodeInfo;
use crate::util::xor_bytes;


#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SignedKadMessage {
    message: String,
    signature: Signature,
    puzzle_x: [u8;20]
}

#[derive(Error, Debug)]
pub enum SignedMessageError {
    #[error("The Node ID does not match the public key or the key is not admissible")]
    C1VerifyError,
    #[error("The message crypto puzzle is not valid")]
    C2VerifyError,
}

impl SignedKadMessage {
    pub(crate) fn sign_serialize(msg: KadMessage, key: &Keypair) -> crate::Result<Self> {
        let s = msg.as_message()?;
        let sig = key.try_sign(s.as_bytes())?;
        let x = Self::dynamic_cryptopuzzle_generate(msg.sender.id());

        Ok(
            SignedKadMessage {
                message: s,
                signature: sig,
                puzzle_x: x
            }
        )
    }

    pub(crate) fn verify_deserialize(self) -> crate::Result<KadMessage> {
        let msg = KadMessage::from_string(&self.message)?;
        let pkey = &msg.sender.pub_key().unwrap();
        pkey.verify(self.message.as_bytes(), &self.signature)?;
        if !msg.sender.id().verify_c1(pkey) {
            return Err(SignedMessageError::C1VerifyError)?;
        }
        if !Self::dynamic_cryptopuzzle_verify(&self.puzzle_x, msg.sender.id()) {
            return Err(SignedMessageError::C2VerifyError)?;
        }
        Ok(msg)
    }

    pub(crate) fn as_message(&self) -> Result<String, KadError> {
        let res = serde_json::to_string(self).map_err(KadError::MessageSerializeError)?;
        assert!(res.len() < MESSAGE_BUFFER_LENGTH);
        Ok(res)
    }

    pub(crate) fn from_string(data: &str) -> Result<Self, KadError> {
        serde_json::from_str(data).map_err(KadError::MessageDeserializeError)
    }

    fn dynamic_cryptopuzzle_generate(id: &KadID) -> [u8;20] {
        loop {
            let x = KadID::random();
            let mut hasher = sha1_smol::Sha1::new();
            hasher.update(&(id ^ &x));
            let p = hasher.digest().bytes();
            if util::get_leading_zeros(&p) > SECURE_KEY_C2 as u8 {
                return x.into_array();
            }
        }
    }

    fn dynamic_cryptopuzzle_verify(x: &[u8;20], id: &KadID) -> bool {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(&xor_bytes(id.as_ref(), x));
        let p = hasher.digest().bytes();
        util::get_leading_zeros(&p) > SECURE_KEY_C2 as u8
    }
}

#[derive(Serialize,Deserialize,Debug)]
pub(crate) struct KadMessage {
    // TODO: Restrict access to fields
    pub(crate) timestamp: OffsetDateTime,
    pub(crate) sender: NodeInfo,
    pub(crate) payload: KadMessagePayload
}

impl KadMessage {
    fn as_message(&self) -> Result<String, KadError> {
        let res = serde_json::to_string(self).map_err(KadError::MessageSerializeError)?;
        Ok(res)
    }

    fn from_string(data: &str) -> Result<Self, KadError> {
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
    Broadcast(Vec<u8>)
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
