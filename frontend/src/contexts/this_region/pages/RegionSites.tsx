import { Container, Heading } from "@chakra-ui/react"
import SitesTable from "../components/SitesTable"
import { useRequiredRegionContext } from "../region_context"
import { RegionDetails } from "../types"

export default function RegionSites() {
  const region: RegionDetails = useRequiredRegionContext()

  return (
    <Container maxWidth={"2xl"}>
      <Heading as="h1" size="2xl">
        This Region: {region.network_id}
      </Heading>

      <Heading as="h2" size="xl" mt={4}>
        Sites
      </Heading>
      <SitesTable />
    </Container>
  )
}
