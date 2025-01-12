import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"
import { RegionDetails } from "./types"

export default class ThisRegionApi extends BaseApi {
  show(): Promise<ApiResult<RegionDetails, any>> {
    return this.apiCall("this_region")
  }

  create(
    name: string,
    description: string,
  ): Promise<ApiResult<RegionDetails, any>> {
    return this.apiCall("this_region/create", "POST", {
      name,
      description,
    })
  }
}
