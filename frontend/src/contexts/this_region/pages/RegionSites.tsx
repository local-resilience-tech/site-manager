import { Container, Heading } from "@chakra-ui/react"
import SitesTable from "../components/SitesTable"

export default function RegionSites() {
  return (
    <Container maxWidth={"2xl"}>
      <Heading as="h1" size="2xl">
        This Region
      </Heading>

      <Heading as="h2" size="xl" mt={4}>
        Sites
      </Heading>
      <SitesTable />
    </Container>
  )
}
