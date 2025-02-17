use anyhow::Result;
use iroh::NodeAddr;
use p2panda_discovery::{BoxedStream, Discovery, DiscoveryEvent};

#[derive(Debug)]
pub struct ManualDiscovery {}

impl ManualDiscovery {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl Discovery for ManualDiscovery {
    fn subscribe(&self, network_id: [u8; 32]) -> Option<BoxedStream<Result<DiscoveryEvent>>> {
        let _ = network_id;
        println!("ManualDiscovery: subscribe");
        None
    }

    fn update_local_address(&self, addr: &NodeAddr) -> Result<()> {
        println!("ManualDiscovery: update_local_address {:#?}", addr);
        Ok(())
    }
}
