import { createContext, useContext } from "react"
import { RegionDetails } from "./types"

const RegionContext = createContext<null | RegionDetails>(null)

export function useRequiredRegionContext(): RegionDetails {
  const context = useContext(RegionContext)
  if (!context) {
    throw new Error("RegionContext not found")
  }
  return context
}

export default RegionContext
