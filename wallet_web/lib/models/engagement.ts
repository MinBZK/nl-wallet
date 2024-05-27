export interface EngagementResponse {
  urls?: EngagementUrls,
}

export interface EngagementUrls {
  status_url: string,
  disclosed_attributes_url: string,
}

export interface EngagementOptions {
  session_type: SessionType,
  usecase: string,
}

export enum SessionType {
  SameDevice = "same_device",
  CrossDevice = "cross_device",
}
