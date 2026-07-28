import { getStatus as apiGetStatus } from "@/api/status"
import { type StatusResponse } from "@/models/openid4vc"
import { vi, type Mock } from "vitest"

const statusResponse: StatusResponse = {
  status: "CREATED",
  ul: new URL("example://app.example.com/-/"),
}

export const getStatus = vi.fn<typeof apiGetStatus>().mockImplementation(async () => statusResponse) as Mock<
  typeof apiGetStatus
>
