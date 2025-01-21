import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"
import { SiteDetails, RegionDetails } from "./types"

export default class ThisSiteApi extends BaseApi {
  show(): Promise<ApiResult<SiteDetails | null, any>> {
    return this.apiCall("this_site")
  }

  create(name: string): Promise<ApiResult<SiteDetails, any>> {
    return this.apiCall("this_site/create", "POST", { name })
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
