import { type AppUL, type SessionType } from "./status"

export type ErrorType = "failed" | "cancelled" | "expired"

export type StatusUrl = string & { __typename: "status_url" }

export type SessionState = "loading" | "created" | "in-progress" | "success" | "error"

export type ModalState =
  | { kind: "loading" }
  | {
      kind: "created"
      ul: AppUL
      statusUrl: StatusUrl
      sessionType: SessionType
      sessionToken: string
    }
  | {
      kind: "in-progress"
      statusUrl: StatusUrl
      sessionType: SessionType
      sessionToken: string
    }
  | { kind: "success"; sessionType: SessionType; sessionToken: string }
  | { kind: "error"; errorType: ErrorType }
