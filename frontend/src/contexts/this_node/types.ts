export type NodeAddr = {
  node_id: string
  info: {
    relay_url: string
    direct_addresses: string[]
  }
}

export type NodeDetails = {
  network_name: string
  panda_node_id: string
  iroh_node_addr: NodeAddr
  peers: NodeAddr[]
}

export type BootstrapPeer = {
  node_id: string
  ip4: string
}
