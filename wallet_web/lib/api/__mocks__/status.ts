import { type AppUL, type StatusResponse } from "@/models/status"
import { vi } from "vitest"

const statusResponse: StatusResponse = {
  status: "CREATED",
  ul: "ul_123" as AppUL,
}

export const getStatus = vi.fn().mockImplementation(async () => statusResponse)
