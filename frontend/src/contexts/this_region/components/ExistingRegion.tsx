import { Box, Text } from "@chakra-ui/react"
import { BootstrapNode, BootstrapNodeData, ThisNodeApi } from "../../this_node"
import { useState } from "react"

const nodeApi = new ThisNodeApi()

export default function ExistingRegion() {
  const [bootstrapData, setBootstrapData] = useState<BootstrapNodeData | null>(
    null,
  )

  const onSubmitBootstrapNode = (data: BootstrapNodeData) => {
    nodeApi.bootstrap(data.node_id, data.ip_address)

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
