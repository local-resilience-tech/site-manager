import { Center, Container, Spinner } from "@chakra-ui/react"
import { useEffect, useState } from "react"

import NewSite, { NewSiteData } from "../components/NewSite"
import { SiteDetails, RegionDetails } from "../types"
import FindRegion from "../components/FindRegion"
import ThisSiteApi from "../api"
import { ApiResult } from "../../shared/types"
import { NewRegionData } from "../components/NewRegion"

type SiteStep = null | "set_name" | "find_region" | "done"

function getStep(
  site: SiteDetails | null,
  region: RegionDetails | null,
): SiteStep {
  if (!site) {
    return "set_name"
  } else if (!region) {
    return "find_region"
  }
  return "done"
}

const api = new ThisSiteApi()

const getSite = async (): Promise<SiteDetails | null> => {
  const result = await api.show()
  if ("Ok" in result) return result.Ok
  return null
}

const getRegion = async (): Promise<RegionDetails | null> => {
  const result = await api.showRegion()
  if ("Ok" in result) return result.Ok
  return null
}

export default function () {
  const [site, setSite] = useState<SiteDetails | null>(null)
  const [region, setRegion] = useState<RegionDetails | null>(null)
  const [step, setStep] = useState<SiteStep>(null)
  const [loading, setLoading] = useState(true)

  const updateSite = (newSite: SiteDetails | null) => {
    console.log("Updating site", newSite)
    setSite(newSite)
  }

  const updateRegion = (newRegion: RegionDetails | null) => {
    console.log("Updating region", newRegion)
    setRegion(newRegion)
  }

  const withLoading = async (fn: () => Promise<void>) => {
    setLoading(true)
    await fn()
    setLoading(false)
  }

  const fetchRegion = async () => {
    withLoading(async () => {
      console.log("EFFECT: fetchRegion")
      const newRegion = await getRegion()
      updateRegion(newRegion)
    })
  }

  const fetchSiteAndRegion = async () => {
    withLoading(async () => {
      console.log("EFFECT: fetchSiteAndRegion")
      const newSite = await getSite()
      const newRegion = newSite ? await getRegion() : null
      updateSite(newSite)
      updateRegion(newRegion)
    })
  }

  const siteUnknown = step == null

  useEffect(() => {
    if (site == null) fetchSiteAndRegion()
  }, [])
  useEffect(() => {
    if (site != null && region == null) fetchRegion()
  }, [siteUnknown])
  useEffect(() => {
    setStep(getStep(site, region))
  }, [site, region])

  const onSubmitNewSite = (data: NewSiteData) => {
    api.create(data.name).then((result: ApiResult<SiteDetails, any>) => {
      if ("Ok" in result) updateSite(result.Ok)
    })
  }

  const onSubmitNewRegion = (data: NewRegionData) => {
    api
      .createRegion(data.name, data.description)
      .then((result: ApiResult<RegionDetails, any>) => {
        if ("Ok" in result) updateRegion(result.Ok)
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
      {step == "set_name" && <NewSite onSubmitNewSite={onSubmitNewSite} />}
      {step == "find_region" && site && (
        <FindRegion site={site} onSubmitNewRegion={onSubmitNewRegion} />
      )}
      {step == "done" && (
        <div>
          <h1>Site created!</h1>
          <p>Site: {site?.name}</p>
          <p>Region: {region?.name}</p>
        </div>
      )}
    </Container>
  )
}
