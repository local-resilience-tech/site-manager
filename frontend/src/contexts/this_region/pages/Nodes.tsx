import { Container, Heading, VStack } from "@chakra-ui/react"
import { useContext, useEffect, useState } from "react"
import { RegionContext } from "../provider_contexts"
import NodesList from "../components/NodesList"
import { NodeDetails } from "../../this_node"
import ThisRegionApi from "../api"
import { Loading, useLoading } from "../../shared"

const api = new ThisRegionApi()

const getNodes = async (): Promise<NodeDetails[] | null> => {
  const result = await api.nodes()
  if ("Ok" in result) return result.Ok
  return null
}

export default function Nodes() {
  const regionDetails = useContext(RegionContext)

  if (!regionDetails) {
    return <Container>No region</Container>
  }

  const [nodes, setNodes] = useState<NodeDetails[] | null>(null)
  const [loading, withLoading] = useLoading(true)

  const fetchNodes = async () => {
    withLoading(async () => {
      const result = await getNodes()
      console.log("EFFECT: fetchNodes", result)
      setNodes(result)
    })
  }

  useEffect(() => {
    if (nodes == null) fetchNodes()
  }, [])

  if (loading) return <Loading />

  return (
    <Container maxWidth={"2xl"}>
      <VStack alignItems={"stretch"}>
        <Heading as="h1" size="2xl">
          {regionDetails.network_id}
        </Heading>
        <Heading as="h2" size="lg">
          Nodes
        </Heading>
        {nodes && <NodesList nodes={nodes} />}
      </VStack>
    </Container>
  )
}
