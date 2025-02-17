use std::time::SystemTime;

use p2panda_core::{
    cbor::{decode_cbor, encode_cbor, DecodeError, EncodeError},
    Body, Header, Operation, PrivateKey, PruneFlag,
};
use p2panda_store::{LocalLogStore, MemoryStore};
use serde::{Deserialize, Serialize};

use super::topics::LogId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extensions {
    log_id: LogId,

    #[serde(rename = "prune", skip_serializing_if = "PruneFlag::is_not_set", default = "PruneFlag::default")]
    prune_flag: PruneFlag,
}

pub async fn create_header(
    store: &mut MemoryStore<LogId, Extensions>,
    log_id: LogId,
    private_key: &PrivateKey,
    maybe_body: Option<Body>,
    prune_flag: bool,
) -> Header<Extensions> {
    let public_key = private_key.public_key();

    let Ok(latest_operation) = store.latest_operation(&public_key, &log_id).await;

    let (seq_num, backlink) = match latest_operation {
        Some((header, _)) => (header.seq_num + 1, Some(header.hash())),
        None => (0, None),
    };

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time from operation system")
        .as_secs();

    let extensions = Extensions {
        log_id,
        prune_flag: PruneFlag::new(prune_flag),
    };

    let mut header = Header {
        version: 1,
        public_key,
        signature: None,
        payload_size: maybe_body.as_ref().map_or(0, |body| body.size()),
        payload_hash: maybe_body.as_ref().map(|body| body.hash()),
        timestamp,
        seq_num,
        backlink,
        previous: vec![],
        extensions: Some(extensions),
    };
    header.sign(private_key);

    header
}

pub fn encode_gossip_message(header: &Header<Extensions>, body: Option<&Body>) -> Result<Vec<u8>, EncodeError> {
    encode_cbor(&(header.to_bytes(), body.map(|body| body.to_bytes())))
}

#[derive(Debug)]
pub struct OperationDetails {
    pub hash: String,
    pub public_key: String,
    pub timestamp: u64,
    pub seq_num: u64,
}

pub fn prepare_for_logging(operation: Operation<Extensions>) -> OperationDetails {
    let Operation { hash, header, body: _ } = operation;
    let header = header.clone();

    return OperationDetails {
        hash: hash.to_string(),
        public_key: header.public_key.to_string(),
        timestamp: header.timestamp,
        seq_num: header.seq_num,
    };
}

pub fn decode_gossip_message(bytes: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>), DecodeError> {
    decode_cbor(bytes)
}
