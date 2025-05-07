import { Container, Heading, VStack } from "@chakra-ui/react"
import { useContext } from "react"
import { RegionContext } from "../provider_contexts"
import SitesList from "../components/SitesList"

export default function Sites() {
  const regionDetails = useContext(RegionContext)

  if (!regionDetails) {
    return <Container>No region</Container>
  }

  return (
    <Container maxWidth={"2xl"}>
      <VStack alignItems={"stretch"}>
        <Heading as="h1" size="2xl">
          {regionDetails.network_id}
        </Heading>
        <Heading as="h2" size="lg">
          Sites
        </Heading>
        <SitesList />
      </VStack>
    </Container>
  )
}
