import { createContext } from "react"
import { RegionDetails } from "./types"

export const RegionContext = createContext<RegionDetails | null>(null)
