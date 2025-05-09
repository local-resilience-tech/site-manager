import { Container, Heading, VStack } from "@chakra-ui/react"
import { useContext, useEffect, useState } from "react"
import { RegionContext } from "../provider_contexts"
import SitesList from "../components/SitesList"
import { SiteDetails } from "../../this_node"
import ThisRegionApi from "../api"
import { Loading, useLoading } from "../../shared"

const api = new ThisRegionApi()

const getSites = async (): Promise<SiteDetails[] | null> => {
  const result = await api.sites()
  if ("Ok" in result) return result.Ok
  return null
}

export default function Sites() {
  const regionDetails = useContext(RegionContext)

  if (!regionDetails) {
    return <Container>No region</Container>
  }

  const [sites, setSites] = useState<SiteDetails[] | null>(null)
  const [loading, withLoading] = useLoading(true)

  const fetchSites = async () => {
    withLoading(async () => {
      const result = await getSites()
      console.log("EFFECT: fetchSites", result)
      setSites(result)
    })
  }

  useEffect(() => {
    if (sites == null) fetchSites()
  }, [])

  if (loading) return <Loading />

  return (
    <Container maxWidth={"2xl"}>
      <VStack alignItems={"stretch"}>
        <Heading as="h1" size="2xl">
          {regionDetails.network_id}
        </Heading>
        <Heading as="h2" size="lg">
          Sites
        </Heading>
        {sites && <SitesList sites={sites} />}
      </VStack>
    </Container>
  )
}
