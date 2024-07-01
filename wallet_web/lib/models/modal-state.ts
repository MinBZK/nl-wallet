import { type AppUL, SessionType } from "./status"

export type ErrorType = "failed" | "cancelled" | "expired"

export type StatusUrl = string & { __typename: "status_url" }

export type ModalState =
  | { kind: "loading" }
  | {
      kind: "created"
      ul: AppUL
      status_url: StatusUrl
      session_type: SessionType
      session_token: string
    }
  | { kind: "in-progress"; status_url: StatusUrl; session_type: SessionType; session_token: string }
  | { kind: "success"; session_type: SessionType; session_token: string }
  | { kind: "error"; error_type: ErrorType }
