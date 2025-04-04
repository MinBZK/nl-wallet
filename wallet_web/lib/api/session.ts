import { type SessionOptions, type SessionResponse } from "@/models/session"
import axios, { AxiosError } from "axios"
import { catch_axios_error, REQUEST_TIMEOUT } from "./base"

export const createSession = async (startUrl: URL, sessionOptions: SessionOptions): Promise<SessionResponse> => {
  try {
    const response = await axios.post(startUrl.toString(), sessionOptions, {
      timeout: REQUEST_TIMEOUT,
    })
    return await response.data
  } catch (e) {
    return catch_axios_error(e as AxiosError)
  }
}
