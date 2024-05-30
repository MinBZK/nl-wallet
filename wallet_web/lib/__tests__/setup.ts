import { config } from "@vue/test-utils"
import { setupJestCanvasMock } from "jest-canvas-mock"
import { vi } from "vitest"
import { isMobileKey } from "../util/projection_keys"

vi.hoisted(() => {
  vi.stubGlobal("jest", vi)
})

config.global.provide = { [isMobileKey as symbol]: false }

setupJestCanvasMock()
