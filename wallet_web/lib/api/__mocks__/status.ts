import { type StatusResponse } from "@/models/openid4vc"
import { vi } from "vitest"

const statusResponse: StatusResponse = {
  status: "CREATED",
  ul: new URL("example://app.example.com/-/"),
}

export const getStatus = vi.fn().mockImplementation(async () => statusResponse)
