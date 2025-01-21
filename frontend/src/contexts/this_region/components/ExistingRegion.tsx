import { Box, Text } from "@chakra-ui/react"
import BootstrapNode, { BootstrapNodeData } from "./BootstrapNode"
import { useState } from "react"
import ThisRegionApi from "../api"

const api = new ThisRegionApi()

export default function ExistingRegion() {
  const [bootstrapData, setBootstrapData] = useState<BootstrapNodeData | null>(
    null,
  )

  const onSubmitBootstrapNode = (data: BootstrapNodeData) => {
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
