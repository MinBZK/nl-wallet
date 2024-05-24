import type { EngagementResponse } from "@/models/engagement"
import { vi } from "vitest"

const engagementResponse: EngagementResponse = {
  urls: {
    status_url: "http://localhost:3001/disclosure/mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl/status",
    disclosed_attributes_url: "http://localhost:3004/disclosure/sessions/mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl/disclosed_attributes"
  }
}

export const createEngagement = vi.fn().mockImplementation(async () => engagementResponse)
