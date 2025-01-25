import { Box, Text } from "@chakra-ui/react"
import {
  BootstrapNode,
  BootstrapNodeData,
  BootstrapPeer,
} from "../../this_node"
import { useState } from "react"
import ThisRegionApi from "../api"

const regionApi = new ThisRegionApi()

export default function ExistingRegion() {
  const [bootstrapData, setBootstrapData] = useState<BootstrapNodeData | null>(
    null,
  )

  const onSubmitBootstrapNode = (data: BootstrapNodeData) => {
    const peer: BootstrapPeer = {
      node_id: data.node_id,
      ip4: data.ip_address,
    }
    regionApi.bootstrap(data.network_name, peer)

    // temp
    setBootstrapData(data)
  }

  if (bootstrapData == null) {
    return <BootstrapNode onSubmit={onSubmitBootstrapNode} />
  }

  return (
    <Box>
      <Text>TODO: We should have booted the network</Text>
    </Box>
  )
}
