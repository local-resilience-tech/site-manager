#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SiteAnnounced {
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum SiteEventPayload {
    SiteAnnounced(SiteAnnounced),
}

#[derive(Debug)]
pub struct SiteEvent {
    payload: SiteEventPayload,
}

impl SiteEvent {
    pub fn new(payload: SiteEventPayload) -> Self {
        SiteEvent { payload }
    }
}
