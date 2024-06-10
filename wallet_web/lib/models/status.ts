export type EngagementUrl = string & { __typename: "engagement_url" }

export type StatusResponse =
  | { status: "CREATED"; engagement_url: EngagementUrl }
  | { status: "WAITING_FOR_RESPONSE" }
  | { status: "DONE" }
  | { status: "FAILED" }
  | { status: "CANCELLED" }
  | { status: "EXPIRED" }

export enum SessionType {
  SameDevice = "same_device",
  CrossDevice = "cross_device",
}
