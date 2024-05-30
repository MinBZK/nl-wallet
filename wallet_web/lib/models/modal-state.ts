import { SessionType } from "./engagement"
import { type EngagementUrl } from "./status"

export type ErrorType = "failed" | "cancelled" | "expired"

export type StatusUrl = string & { __typename: "status_url" }

export type ModalState =
  | { kind: "starting" }
  | {
      kind: "created"
      engagement_url: EngagementUrl
      status_url: StatusUrl
      session_type: SessionType
    }
  | { kind: "success"; session_type: SessionType }
  | { kind: "error"; error_type: ErrorType }
  | { kind: "in_progress" }
