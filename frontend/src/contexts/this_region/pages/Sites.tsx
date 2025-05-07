import { Container, Heading, Table, VStack } from "@chakra-ui/react"

export default function Sites() {
  return (
    <Container maxWidth={"2xl"}>
      <VStack alignItems={"stretch"}>
        <Heading as="h1" size="2xl">
          Sites
        </Heading>
        <Table.Root variant="line">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeader>Name</Table.ColumnHeader>
              <Table.ColumnHeader>Node ID</Table.ColumnHeader>
            </Table.Row>
          </Table.Header>
          <Table.Body>
            <Table.Row>
              <Table.Cell>Site #1</Table.Cell>
              <Table.Cell>XXX</Table.Cell>
            </Table.Row>
          </Table.Body>
        </Table.Root>
      </VStack>
    </Container>
  )
}
