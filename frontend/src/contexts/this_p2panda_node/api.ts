import { BaseApi } from "../shared"
import { ApiResult } from "../shared/types"

export default class ThisP2PandaNodeApi extends BaseApi {
  showNode(): Promise<ApiResult<any, any>> {
    return this.apiCall("this_p2panda_node")
  }

  restart() {
    return this.apiCall("this_p2panda_node/restart", "POST")
  }
}
