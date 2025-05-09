import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"
import { BootstrapPeer } from "../this_p2panda_node"
import { NodeDetails } from "../this_node"
import { RegionDetails } from "./types"

export default class ThisRegionApi extends BaseApi {
  show(): Promise<ApiResult<RegionDetails, any>> {
    return this.apiCall("this_region")
  }

  nodes(): Promise<ApiResult<NodeDetails[], any>> {
    return this.apiCall("this_region/nodes")
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
