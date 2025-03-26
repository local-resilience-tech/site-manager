use p2panda_core::Operation;
use p2panda_node::extensions::NodeExtensions;

#[allow(dead_code)]
#[derive(Debug)]
pub struct OperationDetails {
    pub hash: String,
    pub public_key: String,
    pub timestamp: u64,
    pub seq_num: u64,
}

pub fn prepare_for_logging(operation: Operation<NodeExtensions>) -> OperationDetails {
    let Operation { hash, header, body: _ } = operation;
    let header = header.clone();

    return OperationDetails {
        hash: hash.to_string(),
        public_key: header.public_key.to_string(),
        timestamp: header.timestamp,
        seq_num: header.seq_num,
    };
}
