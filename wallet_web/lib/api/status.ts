import { type StatusResponse } from "@/models/status"
import axios from "axios"

export const getStatus = (absoluteUrl: string): Promise<StatusResponse> =>
  axios.get(absoluteUrl).then((response) => response.data)
