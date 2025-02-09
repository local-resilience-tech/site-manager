import { useEffect, useState } from "react"
import { RegionDetails } from "../types"
import { Center, Container, Spinner } from "@chakra-ui/react"
import ThisRegionApi from "../api"
import SetRegion from "../components/SetRegion"
import { NewRegionData } from "../components/NewRegion"
import { ApiResult } from "../../shared/types"
import RegionContext from "../region_context"

const regionApi = new ThisRegionApi()

const getRegion = async (): Promise<RegionDetails | null> => {
  const result = await regionApi.show()
  if ("Ok" in result) return result.Ok
  return null
}

export default function EnsureRegion({
  children,
}: {
  children: React.ReactNode
}) {
  const [region, setRegion] = useState<RegionDetails | null>(null)
  const [loading, setLoading] = useState(true)

  const updateRegion = (newRegion: RegionDetails | null) => {
    console.log("Updated region", newRegion)
    setRegion(newRegion)
  }

  const withLoading = async (fn: () => Promise<void>) => {
    setLoading(true)
    await fn()
    setLoading(false)
  }

  const fetchRegion = async () => {
    withLoading(async () => {
      const newRegion = await getRegion()
      console.log("EFFECT: fetchRegion", newRegion)
      updateRegion(newRegion || null)
    })
  }

  const onSubmitNewRegion = (data: NewRegionData) => {
    regionApi.bootstrap(data.name, null).then((result: ApiResult<any, any>) => {
      if ("Ok" in result) {
        console.log("Successfully bootstrapped", result, "Fetching region")
        fetchRegion()
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

  if (region == null) {
    return (
      <Container maxWidth={"2xl"}>
        <SetRegion onSubmitNewRegion={onSubmitNewRegion} />
      </Container>
    )
  } else {
    return (
      <RegionContext.Provider value={region}>
        <Container maxWidth={"2xl"}>{children}</Container>
      </RegionContext.Provider>
    )
  }
}
