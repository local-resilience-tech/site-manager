import { Center, Container, Spinner } from "@chakra-ui/react"
import { useEffect, useState } from "react"
import NewSite, { NewSiteData } from "../components/NewSite"
import { SiteDetails } from "../types"
import ThisSiteApi from "../api"
import { ApiResult } from "../../shared/types"

const api = new ThisSiteApi()

const getSite = async (): Promise<SiteDetails | null> => {
  const result = await api.show()
  if ("Ok" in result) return result.Ok
  return null
}

export default function EnsureSite() {
  const [site, setSite] = useState<SiteDetails | null>(null)
  const [loading, setLoading] = useState(true)

  const updateSite = (newSite: SiteDetails | null) => {
    console.log("Updating site", newSite)
    setSite(newSite)
  }

  const withLoading = async (fn: () => Promise<void>) => {
    setLoading(true)
    await fn()
    setLoading(false)
  }

  const fetchSite = async () => {
    withLoading(async () => {
      console.log("EFFECT: fetchSite")
      const newSite = await getSite()
      updateSite(newSite)    })
  }

  useEffect(() => {
    if (site == null) fetchSite()
  }, [])

  const onSubmitNewSite = (data: NewSiteData) => {
    api.create(data.name).then((result: ApiResult<SiteDetails, any>) => {
      if ("Ok" in result) updateSite(result.Ok)
    })
  }

  if (loading) {
    return (
      <Container maxWidth={"2xl"}>
        <Center>
          <Spinner size="lg" opacity={0.5} />
        </Center>
      </Container>
    )
  }

  return (
    <Container maxWidth={"2xl"}>
      {site == null && <NewSite onSubmitNewSite={onSubmitNewSite} />}
      {site != null && (
        <div>
          <h1>Site created!</h1>
          <p>Site: {site?.name}</p>
        </div>
      )}
    </Container>
  )
}
