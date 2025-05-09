import axios, { type AxiosResponse } from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const cancelSession = async (url: URL): Promise<AxiosResponse> =>
  await axios.delete(url.toString(), { timeout: REQUEST_TIMEOUT })
