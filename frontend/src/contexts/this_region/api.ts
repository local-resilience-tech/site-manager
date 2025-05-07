import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"
import { BootstrapPeer } from "../this_node"
import { SiteDetails } from "../this_site"
import { RegionDetails } from "./types"

export default class ThisRegionApi extends BaseApi {
  show(): Promise<ApiResult<RegionDetails, any>> {
    return this.apiCall("this_region")
  }

  sites(): Promise<ApiResult<[SiteDetails], any>> {
    return this.apiCall("this_region/sites")
  }

  bootstrap(
    network_name: string,
    bootstrap_peer: BootstrapPeer | null,
  ): Promise<ApiResult<any, any>> {
    return this.apiCall("this_region/bootstrap", "POST", {
      network_name,
      bootstrap_peer,
    })
  }
}
