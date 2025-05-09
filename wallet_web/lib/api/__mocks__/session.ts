import { type SessionResponse } from "@/models/session"
import { vi } from "vitest"

const sessionResponse: SessionResponse = {
  status_url: new URL("http://localhost:3001/disclosure/mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl/status"),
  session_token: "mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl",
}

export const createSession = vi.fn().mockResolvedValue(sessionResponse)
