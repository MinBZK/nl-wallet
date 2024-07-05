import { type SessionOptions, type SessionResponse } from "@/models/session"
import axios from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const createSession = async (
  baseUrl: string,
  sessionOptions: SessionOptions,
): Promise<SessionResponse> => {
  const response = await axios.post(new URL("sessions", baseUrl).toString(), sessionOptions, {
    timeout: REQUEST_TIMEOUT,
  })
  return await response.data
}
