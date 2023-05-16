#![allow(dead_code)]

mod rpc;
pub mod node;
pub mod kadid;
mod buckets;
mod util;


use thiserror::Error;
use tokio::io;


type Result<T> = anyhow::Result<T>;

/// The max bucket size for a given node
const K: u8 = 10;

/// The concurrency factor
const ALPHA: u8 = 2;

/// Max buffer length
const MESSAGE_BUFFER_LENGTH: usize = 4096;

/// Bucket refresh interval
/// 3600s = 1 hour
const BUCKET_REFRESH_INTERVAL: i64 = 3600;

/// RPC Request timeout
/// 300s = 5min
const REQUEST_TIMEOUT: i64 = 300;

const MAX_MESSAGE_BUFFER: usize = 4096;

const SECURE_KEY_C1: usize = 1;
const SECURE_KEY_C2: usize = 2;

const BROADCAST_CACHE_LIMIT: u64 = 4096;
const BROADCAST_AGE_LIMIT_SECS: u64 = 300;

#[derive(Error, Debug)]
pub enum KadError {
    #[error("Failed to deserialize Node data")]
    LocalDeserializeError(serde_json::Error),
    #[error("Failed to serialize protocol message")]
    MessageSerializeError(serde_json::Error),
    #[error("Failed to deserialize protocol message")]
    MessageDeserializeError(serde_json::Error),
    #[error("Generic async IO error")]
    IOError(io::Error)
}