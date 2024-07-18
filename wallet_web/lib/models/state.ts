import { type AppUL, type SessionType } from "./status"

export type ErrorType = "failed" | "cancelled" | "expired"

export type StatusUrl = string & { __typename: "status_url" }

export type Session = {
  statusUrl: StatusUrl
  sessionType: SessionType
  sessionToken: string
}

export type ModalState =
  | { kind: "creating" }
  | {
      kind: "created"
      ul: AppUL
      session: Session
    }
  | { kind: "loading"; session: Session }
  | {
      kind: "in-progress"
      session: Session
    }
  | { kind: "success"; session: Session }
  | { kind: "error"; errorType: ErrorType; session?: Session }
  | { kind: "confirm-stop"; prev: ModalState; session: Session }
