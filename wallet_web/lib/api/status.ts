import { type SessionType, type StatusResponse } from "@/models/status"
import axios from "axios"
import { REQUEST_TIMEOUT } from "./base"

export const getStatus = (absoluteUrl: string, sessionType: SessionType): Promise<StatusResponse> =>
  axios
    .get(absoluteUrl, { params: { session_type: sessionType }, timeout: REQUEST_TIMEOUT })
    .then((response) => response.data)
