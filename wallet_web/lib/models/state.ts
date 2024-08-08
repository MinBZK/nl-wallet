import { type AppUL, type SessionType } from "./status"

const errors = ["failed", "cancelled", "expired", "network"] as const
export type ErrorType = (typeof errors)[number]

export const isError = (e: any): e is ErrorType => errors.includes(e)

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
