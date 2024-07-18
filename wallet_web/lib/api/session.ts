import { type SessionOptions, type SessionResponse } from "@/models/session"
import axios, { AxiosError } from "axios"
import { catch_axios_error, REQUEST_TIMEOUT } from "./base"

export const createSession = async (
  baseUrl: string,
  sessionOptions: SessionOptions,
): Promise<SessionResponse> => {
  try {
    const response = await axios.post(new URL("sessions", baseUrl).toString(), sessionOptions, {
      timeout: REQUEST_TIMEOUT,
    })
    return await response.data
  } catch (e) {
    return catch_axios_error(e as AxiosError)
  }
}
