import { type SessionOptions, type SessionResponse } from "@/models/session"
import axios from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const createSession = async (
  baseUrl: string,
  session_options: SessionOptions,
): Promise<SessionResponse> => {
  const response = await axios.post(new URL("sessions", baseUrl).toString(), session_options, {
    timeout: REQUEST_TIMEOUT,
  })
  return await response.data
}
