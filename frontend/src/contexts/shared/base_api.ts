import { ApiResult, OkResult } from "./types"

const headers = {
  "Content-Type": "application/json",
  Accept: "application/json",
}

function getApiHost(): string {
  return import.meta.env.VITE_API_HOST || "/api"
}

export default class BaseApi {
  base_url: string

  constructor() {
    this.base_url = getApiHost()
  }

  async apiCall(
    path: string,
    method: string = "GET",
    body?: object,
  ): Promise<ApiResult<any, any>> {
    try {
      const response = await fetch(`${this.base_url}/${path}`, {
        method,
        headers,
        body: (body && JSON.stringify(body)) || undefined,
      })
      console.log("got response", response)
      if (response.ok) {
        const json = await response.json()
        return {
          Ok: json,
        }
      } else {
        if (response.status == 404) {
          return {
            Ok: null,
          }
        }
      }
      throw new Error(`${response.status}: ${response.statusText}`)
    } catch (error) {
      console.error("Failed to connect to API: ", error)
      return {
        Err: error,
      }
    }
  }
}
