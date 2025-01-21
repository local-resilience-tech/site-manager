import { VStack, Text, Table, Box } from "@chakra-ui/react"
import { useEffect, useState } from "react"
import ThisSiteApi from "../api"
import { NodeDetails } from "../types"

const api = new ThisSiteApi()

const getNode = async (): Promise<NodeDetails | null> => {
  const result = await api.showNode()
  if ("Ok" in result) return result.Ok
  return null
}

export default function ThisNode() {
  const [node, setNode] = useState<NodeDetails | null>(null)

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

  return (
    <VStack alignItems={"stretch"}>
      <Text textStyle="xl">This P2Panda Node</Text>
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
