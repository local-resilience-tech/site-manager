use p2panda_core::PruneFlag;
use serde::{Deserialize, Serialize};

use super::topics::LogId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Extensions {
    log_id: LogId,

    #[serde(rename = "prune", skip_serializing_if = "PruneFlag::is_not_set", default = "PruneFlag::default")]
    prune_flag: PruneFlag,
}
