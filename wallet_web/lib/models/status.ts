export type StatusResponse =
  | { status: "CREATED"; ul: URL } // TODO apparently the UL parameter is optional in the Rust code
  | { status: "WAITING_FOR_RESPONSE" }
  | { status: "DONE" }
  | { status: "FAILED" }
  | { status: "CANCELLED" }
  | { status: "EXPIRED" }

export type SessionType = "same_device" | "cross_device"
