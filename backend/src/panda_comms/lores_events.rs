#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct NodeAnnounced {
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum LoResEventPayload {
    NodeAnnounced(NodeAnnounced),
}

#[derive(Debug)]
pub struct LoResEventHeader {
    pub author_node_id: String,
}

#[derive(Debug)]
pub struct LoResEvent {
    pub header: LoResEventHeader,
    pub payload: LoResEventPayload,
}

impl LoResEvent {
    pub fn new(header: LoResEventHeader, payload: LoResEventPayload) -> Self {
        LoResEvent { header, payload }
    }
}
