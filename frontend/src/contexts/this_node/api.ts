import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"
import { NodeDetails, RegionDetails } from "./types"

export default class ThisNodeApi extends BaseApi {
  show(): Promise<ApiResult<NodeDetails | null, any>> {
    return this.apiCall("this_node")
  }

  create(name: string): Promise<ApiResult<NodeDetails, any>> {
    return this.apiCall("this_node/create", "POST", { name })
  }

  showRegion(): Promise<ApiResult<RegionDetails, any>> {
    return this.apiCall("this_region")
  }

  createRegion(
    name: string,
    description: string,
  ): Promise<ApiResult<RegionDetails, any>> {
    return this.apiCall("this_region/create", "POST", {
      name,
      description,
    })
  }
}
