import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"

export default class ThisNodeApi extends BaseApi {
  showNode(): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node")
  }

  bootstrap(
    network_name: string,
    node_id: string,
    ip_address: string,
  ): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node/bootstrap", "POST", {
      network_name,
      node_id,
      ip_address,
    })
  }
}
