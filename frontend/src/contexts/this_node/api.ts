import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"
import { BootstrapPeer } from "./types"

export default class ThisNodeApi extends BaseApi {
  showNode(): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node")
  }

  bootstrap(
    network_name: string,
    bootstrap_peer: BootstrapPeer,
  ): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node/bootstrap", "POST", {
      network_name,
      bootstrap_peer,
    })
  }
}
