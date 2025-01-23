import { useEffect, useState } from "react"
import { RegionDetails } from "../types"
import { Center, Container, Spinner } from "@chakra-ui/react"
import ThisRegionApi from "../api"
import SetRegion from "../components/SetRegion"
import { NewRegionData } from "../components/NewRegion"
import { ApiResult } from "../../shared/types"
import { ThisNodeApi } from "../../this_node"

const regionApi = new ThisRegionApi()
const nodeApi = new ThisNodeApi()

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
  const [networkId, setNetworkId] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const updateRegion = (newRegion: RegionDetails | null) => {
    console.log("Updating region", newRegion)
    setNetworkId(newRegion?.network_id || null)
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
    regionApi
      .create(data.name)
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
      {networkId == null && <SetRegion onSubmitNewRegion={onSubmitNewRegion} />}
      {networkId != null && children}
    </Container>
  )
}
