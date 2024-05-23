export interface StatusResponse {
  status: Status;
  engagement_url?: string;
}

export enum Status {
  Created = "CREATED",
  WaitingForResponse = "WAITING_FOR_RESPONSE",
  Done = "DONE",
  Failed = "FAILED",
  Cancelled = "CANCELLED",
  Expired = "EXPIRED",
}
