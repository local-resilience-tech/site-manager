import { Table } from "@chakra-ui/react"

export default function SitesTable() {
  return (
    <Table.Root variant="line" mt={4}>
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeader>Name</Table.ColumnHeader>
          <Table.ColumnHeader>ID</Table.ColumnHeader>
        </Table.Row>
      </Table.Header>
    </Table.Root>
  )
}
