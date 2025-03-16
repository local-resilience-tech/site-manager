use anyhow::Result;
use p2panda_core::cbor::{encode_cbor, EncodeError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<Payload> {
    pub payload: Payload,
}

impl<Payload: Serialize> Message<Payload> {
    pub fn encode(payload: Payload) -> Result<Vec<u8>, EncodeError> {
        let message = Message { payload };
        encode_cbor(&message)
    }
}
