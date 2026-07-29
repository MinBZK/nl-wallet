import { createSession as apiCreateSession } from "@/api/session"
import { type SessionResponse } from "@/models/relying_party"
import { vi, type Mock } from "vitest"

const sessionResponse: SessionResponse = {
  status_url: new URL("http://localhost:3001/disclosure/mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl/status"),
  session_token: "mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl",
}

export const createSession = vi.fn<typeof apiCreateSession>().mockResolvedValue(sessionResponse) as Mock<
  typeof apiCreateSession
>
