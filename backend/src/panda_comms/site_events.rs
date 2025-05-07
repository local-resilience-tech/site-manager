#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SiteAnnounced {
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum SiteEventPayload {
    SiteAnnounced(SiteAnnounced),
}
