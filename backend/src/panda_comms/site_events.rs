#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SiteAnnounced {
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum SiteEventPayload {
    SiteAnnounced(SiteAnnounced),
}

#[derive(Debug)]
pub struct SiteEventHeader {
    pub author_node_id: String,
}

#[derive(Debug)]
pub struct SiteEvent {
    pub header: SiteEventHeader,
    pub payload: SiteEventPayload,
}

impl SiteEvent {
    pub fn new(header: SiteEventHeader, payload: SiteEventPayload) -> Self {
        SiteEvent { header, payload }
    }
}
