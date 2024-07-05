import { type AppUL, SessionType } from "./status"

export type ErrorType = "failed" | "cancelled" | "expired"

export type StatusUrl = string & { __typename: "status_url" }

export enum SessionState {
  Loading = "loading",
  Created = "created",
  InProgress = "in-progress",
  Success = "success",
  Error = "error",
}

export type ModalState =
  | { kind: SessionState.Loading }
  | {
      kind: SessionState.Created
      ul: AppUL
      statusUrl: StatusUrl
      sessionType: SessionType
      sessionToken: string
    }
  | {
      kind: SessionState.InProgress
      statusUrl: StatusUrl
      sessionType: SessionType
      sessionToken: string
    }
  | { kind: SessionState.Success; sessionType: SessionType; sessionToken: string }
  | { kind: SessionState.Error; errorType: ErrorType }
