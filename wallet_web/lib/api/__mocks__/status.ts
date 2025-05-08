import { type StatusResponse } from "@/models/status"
import { vi } from "vitest"

const statusResponse: StatusResponse = {
  status: "CREATED",
  ul: new URL("example://app.example.com/-/"),
}

export const getStatus = vi.fn().mockImplementation(async () => statusResponse)
