use p2panda_core::{Extension, Header, PruneFlag};
use serde::{Deserialize, Serialize};

/// Every site_manager peer writes to one single log per topic which is identified by the node's public
/// key and the topic id.
pub type LogId = [u8; 32];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomExtensions {
    pub log_id: LogId,

    #[serde(rename = "prune", skip_serializing_if = "PruneFlag::is_not_set", default = "PruneFlag::default")]
    pub prune_flag: PruneFlag,
}

impl Extension<LogId> for CustomExtensions {
    fn extract(header: &Header<Self>) -> Option<LogId> {
        let Some(extensions) = header.extensions.as_ref() else {
            return None;
        };

        Some(extensions.log_id.clone())
    }
}

impl Extension<PruneFlag> for CustomExtensions {
    fn extract(header: &Header<Self>) -> Option<PruneFlag> {
        let Some(extensions) = header.extensions.as_ref() else {
            return None;
        };

        Some(extensions.prune_flag.clone())
    }
}
