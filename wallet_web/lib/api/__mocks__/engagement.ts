import { type EngagementResponse } from "@/models/engagement"
import { type StatusUrl } from "@/models/modal-state"
import { vi } from "vitest"

const engagementResponse: EngagementResponse = {
  urls: {
    status_url:
      "http://localhost:3001/disclosure/mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl/status" as StatusUrl,
  },
}

export const createEngagement = vi.fn().mockResolvedValue(engagementResponse)
