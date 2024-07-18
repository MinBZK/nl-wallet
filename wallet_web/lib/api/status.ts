import { type SessionType, type StatusResponse } from "@/models/status"
import axios from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const getStatus = async (
  absoluteUrl: string,
  sessionType: SessionType,
): Promise<StatusResponse> => {
  const response = await axios.get(absoluteUrl, {
    params: { session_type: sessionType },
    timeout: REQUEST_TIMEOUT,
  })
  return await response.data
}
