use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<Payload> {
    pub payload: Payload,
}
