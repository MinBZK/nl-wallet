import { type StatusUrl } from "./state"

export interface SessionResponse {
  status_url: StatusUrl
  session_token: string
}

export interface SessionOptions {
  usecase: string
}
