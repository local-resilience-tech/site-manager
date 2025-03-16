import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"

export default class ThisNodeApi extends BaseApi {
  showNode(): Promise<ApiResult<any, any>> {
    return this.apiCall("this_node")
  }

  restart() {
    return this.apiCall("this_node/restart", "POST")
  }
}
