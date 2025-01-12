import { useEffect, useState } from "react"
import { RegionDetails } from "../types"
import { Box } from "@chakra-ui/react"
import ThisRegionApi from "../api"

const api = new ThisRegionApi()

const getRegion = async (): Promise<RegionDetails | null> => {
  const result = await api.show()
  if ("Ok" in result) return result.Ok
  return null
}

export default function () {
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

  useEffect(() => {
    fetchRegion()
  }, [])

  return <Box>TODO: This Region</Box>
}