import axios, { type AxiosResponse } from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const cancelSession = async (absoluteUrl: string): Promise<AxiosResponse> =>
  await axios.delete(absoluteUrl, { timeout: REQUEST_TIMEOUT })
