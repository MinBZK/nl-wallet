export type AppUL = string & { __typename: "app_ul" }

export type StatusResponse =
  | { status: "CREATED"; ul: AppUL }
  | { status: "WAITING_FOR_RESPONSE" }
  | { status: "DONE" }
  | { status: "FAILED" }
  | { status: "CANCELLED" }
  | { status: "EXPIRED" }

export enum SessionType {
  SameDevice = "same_device",
  CrossDevice = "cross_device",
}
