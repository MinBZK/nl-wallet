import axios from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const cancelSession = async (absoluteUrl: string): Promise<() => any> =>
  await axios.delete(absoluteUrl, { timeout: REQUEST_TIMEOUT })
