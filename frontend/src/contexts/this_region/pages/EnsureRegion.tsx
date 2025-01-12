import { useEffect, useState } from "react"
import { RegionDetails } from "../types"
import { Box, Center, Container, Spinner } from "@chakra-ui/react"
import ThisRegionApi from "../api"
import FindRegion from "../components/FindRegion"
import { NewRegionData } from "../components/NewRegion"
import { ApiResult } from "../../shared/types"

const api = new ThisRegionApi()

const getRegion = async (): Promise<RegionDetails | null> => {
  const result = await api.show()
  if ("Ok" in result) return result.Ok
  return null
}

export default function EnsureRegion({ children }: { children: React.ReactNode }) {
  const [region, setRegion] = useState<RegionDetails | null>(null)
  const [loading, setLoading] = useState(true)

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

  const onSubmitNewRegion = (data: NewRegionData) => {
    api
      .create(data.name, data.description)
      .then((result: ApiResult<RegionDetails, any>) => {
        if ("Ok" in result) updateRegion(result.Ok)
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
      {region == null && (
        <FindRegion onSubmitNewRegion={onSubmitNewRegion} />
      )}
      {region != null && children}
    </Container>
  )
}