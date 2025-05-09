import { type SessionType, type StatusResponse } from "@/models/status"
import axios, { AxiosError } from "axios"
import { catch_axios_error, REQUEST_TIMEOUT } from "./base"

export const getStatus = async (url: URL, sessionType: SessionType): Promise<StatusResponse> => {
  try {
    const response = await axios.get(url.toString(), {
      params: { session_type: sessionType },
      timeout: REQUEST_TIMEOUT,
    })
    return await response.data
  } catch (e) {
    return catch_axios_error(e as AxiosError)
  }
}
