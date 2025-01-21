export type SiteDetails = {
  id: string
  name: string
}

export type RegionDetails = {
  id: string
  name: string
  description: string
}

export type NodeAddr = {
  node_id: string,
  info: {
    relay_url: string,
    direct_addresses: string[]
  }
}

export type NodeDetails = {
  panda_node_id: string
  iroh_node_addr: NodeAddr
  peers: NodeAddr[]
}
