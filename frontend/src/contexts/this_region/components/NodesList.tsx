import { Table } from "@chakra-ui/react"
import { NodeDetails } from "../../this_node"

export default function NodesList({ nodes: nodes }: { nodes: NodeDetails[] }) {
  return (
    <Table.Root variant="line">
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeader>Name</Table.ColumnHeader>
          <Table.ColumnHeader>Node ID</Table.ColumnHeader>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {nodes.map((node) => (
          <Table.Row key={node.id}>
            <Table.Cell>{node.name}</Table.Cell>
            <Table.Cell>{node.id}</Table.Cell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  )
}
