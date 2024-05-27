import { getStatus } from "@/api/status"
import DeviceChoice from "@/components/DeviceChoice.vue"
import QrCode from "@/components/QrCode.vue"
import WalletModal from "@/components/WalletModal.vue"
import { flushPromises, mount } from "@vue/test-utils"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { nextTick } from "vue"

await import("../setup")

vi.mock("@/api/engagement")
vi.mock("@/api/status")

describe("WalletModal", () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  it("should show qr code directly for desktop mode", async () => {
    const wrapper = mount(WalletModal, { props: { baseUrl: "http://localhost" } })
    await flushPromises()
    const qr = wrapper.getComponent(QrCode)

    expect(qr.text()).toContain("Scan de QR-code met je NL Wallet app")
  })

  it("should refresh qr code", async () => {
    vi.clearAllMocks()

    const status = vi.mocked(getStatus)
    const wrapper = mount(WalletModal, { props: { baseUrl: "http://localhost", pollIntervalInMs: 10 } })
    await flushPromises()
    await nextTick()
    const qr = wrapper.getComponent(QrCode)

    vi.advanceTimersByTime(30)

    expect(qr.text()).toContain("Scan de QR-code met je NL Wallet app")

    expect(status.mock.calls.length).toBe(4)
  })

  it("should ask where the wallet is for mobile mode", async () => {
    happyDOM.settings.navigator.userAgent =
      "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Mobile/15E148 Safari/604.1"

    const wrapper = mount(WalletModal, { props: { baseUrl: "http://localhost" } })
    await flushPromises()
    const choice = wrapper.getComponent(DeviceChoice)

    expect(choice.text()).toContain("Op welk apparaat staat je NL Wallet app?")
  })
})
