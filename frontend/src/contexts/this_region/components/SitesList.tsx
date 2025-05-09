import { Table } from "@chakra-ui/react"
import { SiteDetails } from "../../this_node"

export default function SitesList({ sites }: { sites: SiteDetails[] }) {
  return (
    <Table.Root variant="line">
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeader>Name</Table.ColumnHeader>
          <Table.ColumnHeader>Node ID</Table.ColumnHeader>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {sites.map((site) => (
          <Table.Row key={site.id}>
            <Table.Cell>{site.name}</Table.Cell>
            <Table.Cell>{site.id}</Table.Cell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  )
}
