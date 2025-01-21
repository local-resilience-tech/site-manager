import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"

export default class ThisNodeApi extends BaseApi {
  showNode(): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node")
  }

  bootstrap(node_id: string, ip_address: string): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node/bootstrap", "POST", {
      node_id,
      ip_address,
    })
  }
}
