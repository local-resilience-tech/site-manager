import { VStack, Text, Table, Box } from "@chakra-ui/react"
import { useEffect, useState } from "react"
import ThisP2PandaNodeApi from "../api"
import { P2PandaNodeDetails } from "../types"
import { Button } from "../../../components"

const api = new ThisP2PandaNodeApi()

const getNode = async (): Promise<P2PandaNodeDetails | null> => {
  const result = await api.showNode()
  if ("Ok" in result) return result.Ok
  return null
}

export default function ThisP2PandaNode() {
  const [node, setNode] = useState<P2PandaNodeDetails | null>(null)

  const fetchNode = async () => {
    const node = await getNode()
    console.log("fetched node", node)
    setNode(node)
  }

  useEffect(() => {
    fetchNode()
  }, [])

  if (!node) {
    return <></>
  }

  const restartNode = () => async () => {
    console.log("restarting node")
    await api.restart()
    fetchNode()
  }

  return (
    <VStack alignItems={"stretch"}>
      <Text textStyle="xl">This P2Panda Node</Text>
      <Box mb={4}>
        <Button onClick={restartNode()}>Restart Node</Button>
      </Box>
      <Table.Root variant="line">
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeader>Key</Table.ColumnHeader>
            <Table.ColumnHeader>Value</Table.ColumnHeader>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          <Table.Row>
            <Table.Cell>Panda Node Id</Table.Cell>
            <Table.Cell>
              <pre>{node.panda_node_id}</pre>
            </Table.Cell>
          </Table.Row>
          <Table.Row>
            <Table.Cell>Iroh Node Addr</Table.Cell>
            <Table.Cell>
              <Box maxW={"md"}>
                <pre>{JSON.stringify(node.iroh_node_addr, null, 2)}</pre>
              </Box>
            </Table.Cell>
          </Table.Row>
          <Table.Row>
            <Table.Cell>Peers</Table.Cell>
            <Table.Cell>
              <Box maxW={"md"}>
                <pre>{JSON.stringify(node.peers, null, 2)}</pre>
              </Box>
            </Table.Cell>
          </Table.Row>
        </Table.Body>
      </Table.Root>
    </VStack>
  )
}
