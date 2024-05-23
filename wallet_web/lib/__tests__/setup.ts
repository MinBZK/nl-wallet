import { setupJestCanvasMock } from "jest-canvas-mock"
import { vi } from "vitest"

vi.hoisted(() => {
  vi.stubGlobal("jest", vi)
})

setupJestCanvasMock()
