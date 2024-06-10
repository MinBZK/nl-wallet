import { type SessionOptions, type SessionResponse } from "@/models/session"
import axios from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const createSession = async (
  baseURL: string,
  session_options: SessionOptions,
): Promise<SessionResponse> => {
  const response = await axios.post(`${baseURL}/sessions`, session_options, {
    timeout: REQUEST_TIMEOUT,
  })
  return await response.data
}
