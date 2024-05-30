import { type EngagementUrl, type StatusResponse } from "@/models/status"
import { vi } from "vitest"

const statusResponse: StatusResponse = {
  status: "CREATED",
  engagement_url: "engagement_url_123" as EngagementUrl,
}

export const getStatus = vi.fn().mockImplementation(async () => statusResponse)
