export type NodeAddr = {
  node_id: string
  info: {
    relay_url: string
    direct_addresses: string[]
  }
}

export type P2PandaNodeDetails = {
  panda_node_id: string
  iroh_node_addr: NodeAddr
  peers: NodeAddr[]
}

export type BootstrapPeer = {
  node_id: string
}
