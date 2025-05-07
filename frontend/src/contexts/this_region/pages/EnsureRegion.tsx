import { useEffect, useState } from "react"
import { RegionDetails } from "../types"
import { Center, Container, Spinner } from "@chakra-ui/react"
import ThisRegionApi from "../api"
import SetRegion from "../components/SetRegion"
import { NewRegionData } from "../components/NewRegion"
import { ApiResult } from "../../shared/types"
import { Outlet } from "react-router-dom"
import { RegionContext } from "../provider_contexts"

const regionApi = new ThisRegionApi()

const getRegion = async (): Promise<RegionDetails | null> => {
  const result = await regionApi.show()
  if ("Ok" in result) return result.Ok
  return null
}

export default function EnsureRegion({
  children,
}: {
  children?: React.ReactNode
}) {
  const [regionDetails, setRegionDetails] = useState<RegionDetails | null>(null)
  const [loading, setLoading] = useState(true)

  const withLoading = async (fn: () => Promise<void>) => {
    setLoading(true)
    await fn()
    setLoading(false)
  }

  const fetchRegion = async () => {
    withLoading(async () => {
      const newRegion = await getRegion()
      console.log("EFFECT: fetchRegion", newRegion)
      setRegionDetails(newRegion)
    })
  }

  const onSubmitNewRegion = (data: NewRegionData) => {
    regionApi.bootstrap(data.name, null).then((result: ApiResult<any, any>) => {
      if ("Ok" in result) {
        const newRegion: RegionDetails = {
          network_id: result.Ok.id,
        }
        setRegionDetails(newRegion)
      } else {
        console.log("Failed to bootstrap", result)
      }
    })
  }

  useEffect(() => {
    fetchRegion()
  }, [])

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
      {regionDetails == null && (
        <SetRegion onSubmitNewRegion={onSubmitNewRegion} />
      )}
      <RegionContext.Provider value={regionDetails}>
        {regionDetails != null && (children || <Outlet />)}
      </RegionContext.Provider>
    </Container>
  )
}
