import { createSession } from "@/api/session"
import { getStatus } from "@/api/status"
import DeviceChoice from "@/components/DeviceChoice.vue"
import QrCode from "@/components/QrCode.vue"
import WalletModal from "@/components/WalletModal.vue"
import { translations, translationsKey } from "@/util/translations"
import { isMobileKey } from "@/util/useragent"
import { flushPromises, mount, VueWrapper } from "@vue/test-utils"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { type AppUL } from "../../models/status"
import type { ErrorType } from "@/models/state"

await import("../setup")

vi.mock("@/api/cancel")
vi.mock("@/api/session")
vi.mock("@/api/status")

describe("WalletModal", () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.clearAllTimers()
  })

  it("should show loading screen", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })

    expect(wrapper.find("[data-testid=loading]").exists()).toBe(true)
    await vi.advanceTimersToNextTimerAsync()
    expect(wrapper.find("[data-testid=loading]").exists()).toBe(false)
  })

  it("should show qr code directly for desktop mode", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: {
          [isMobileKey as symbol]: false,
          [translationsKey as symbol]: translations("nl"),
        },
      },
    })
    await vi.advanceTimersToNextTimerAsync()
    expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
  })

  it("should show loading screen after choosing", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: { [isMobileKey as symbol]: true, [translationsKey as symbol]: translations("nl") },
      },
    })

    await vi.advanceTimersToNextTimerAsync()
    const button = wrapper.find("[data-testid=cross_device_button]")
    expect(button.exists()).toBe(true)

    await vi.mocked(getStatus).withImplementation(
      async () => {
        await new Promise((r) => setTimeout(r, 10000))
        return {
          status: "CREATED",
          ul: "ul_456" as AppUL,
        }
      },
      async () => {
        await button.trigger("click")

        expect(wrapper.find("[data-testid=loading]").exists()).toBe(true)
        await vi.advanceTimersToNextTimerAsync()
        await vi.advanceTimersToNextTimerAsync()
        expect(wrapper.find("[data-testid=loading]").exists()).toBe(false)

        expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
      },
    )
  })

  it("should show qr code for mobile after choosing", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: { [isMobileKey as symbol]: true, [translationsKey as symbol]: translations("nl") },
      },
    })
    await vi.advanceTimersToNextTimerAsync()
    await wrapper.find("[data-testid=cross_device_button]").trigger("click")

    await vi.advanceTimersToNextTimerAsync()
    expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
  })

  it("should show loading when closing model", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: {
          [translationsKey as symbol]: translations("nl"),
        },
      },
    })
    await vi.advanceTimersToNextTimerAsync()
    expect(wrapper.find("[data-testid=loading]").exists()).toBe(false)
    await wrapper.find("[data-testid=close_button]").trigger("click")
    await vi.advanceTimersToNextTimerAsync()
    expect(wrapper.find("[data-testid=loading]").exists()).toBe(true)
  })

  it("should refresh qr code", async () => {
    vi.clearAllMocks()

    const status = vi.mocked(getStatus)
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    await flushPromises()
    const qr = wrapper.getComponent(QrCode)

    // twice needed because of "focus-hack"
    await vi.advanceTimersToNextTimerAsync()
    await vi.advanceTimersToNextTimerAsync()
    expect(qr.find("[data-testid=qr]").exists()).toBe(true)

    expect(status.mock.calls.length).toBe(2)
  })

  it("should show in progress when qr code is scanned", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    await flushPromises()
    const qr = wrapper.getComponent(QrCode)
    expect(qr.find("[data-testid=qr]").exists()).toBe(true)

    await vi.mocked(getStatus).withImplementation(
      async () => ({ status: "WAITING_FOR_RESPONSE" }),
      async () => {
        // twice needed because of "focus-hack"
        await vi.advanceTimersToNextTimerAsync()
        await vi.advanceTimersToNextTimerAsync()
        await vi.waitFor(() => {
          expect(wrapper.find("[data-testid=in_progress]").exists()).toBe(true)
        })
      },
    )
  })

  it("should show confirm stop when clicking stop on in-progress screen", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    await flushPromises()
    const qr = wrapper.getComponent(QrCode)
    expect(qr.find("[data-testid=qr]").exists()).toBe(true)

    await vi.mocked(getStatus).withImplementation(
      async () => ({ status: "WAITING_FOR_RESPONSE" }),
      async () => {
        // twice needed because of "focus-hack"
        await vi.advanceTimersToNextTimerAsync()
        await vi.advanceTimersToNextTimerAsync()
        await vi.waitFor(async () => {
          expect(wrapper.find("[data-testid=in_progress]").exists()).toBe(true)
          await wrapper.find("[data-testid=cancel_button]").trigger("click")
          await vi.advanceTimersToNextTimerAsync()
          expect(wrapper.find("[data-testid=confirm_stop]").exists()).toBe(true)
          expect(wrapper.find("[data-testid=in_progress]").exists()).toBe(false)
          // back button should just go back
          await wrapper.find("[data-testid=back_button]").trigger("click")
          expect(wrapper.find("[data-testid=in_progress]").exists()).toBe(true)
        })
      },
    )
  })

  it("should ask where the wallet is for mobile mode", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: { [isMobileKey as symbol]: true, [translationsKey as symbol]: translations("nl") },
      },
    })
    await flushPromises()
    const choice = wrapper.getComponent(DeviceChoice)

    expect(choice.find("[data-testid=device_choice]").exists()).toBe(true)
  })

  it("should have anchor for same device flow", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: { [isMobileKey as symbol]: true, [translationsKey as symbol]: translations("nl") },
      },
    })
    await flushPromises()

    const sameDeviceButton = wrapper.find("[data-testid=same_device_button]")
    expect(sameDeviceButton.attributes("href")).toEqual("ul_123")
  })

  describe("error screens for status", () => {
    let wrapper: VueWrapper

    beforeEach(async () => {
      wrapper = mount(WalletModal, {
        props: {
          startUrl: new URL("http://localhost/sessions"),
          usecase: "test123",
          helpBaseUrl: new URL("https://example.com"),
        },
        global: { provide: { [translationsKey as symbol]: translations("nl") } },
      })
      await flushPromises()
      expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
    })

    it("should show error for expired state", async () => {
      vi.mocked(getStatus).mockResolvedValueOnce({ status: "EXPIRED" })
      // twice needed because of "focus-hack"
      await vi.advanceTimersToNextTimerAsync()
      await vi.advanceTimersToNextTimerAsync()
      expect(wrapper.find("[data-testid=expired_header]").exists()).toBe(true)
    })

    it("should show error for canceled state", async () => {
      vi.mocked(getStatus).mockResolvedValueOnce({ status: "CANCELLED" })
      // twice needed because of "focus-hack"
      await vi.advanceTimersToNextTimerAsync()
      await vi.advanceTimersToNextTimerAsync()
      expect(wrapper.find("[data-testid=cancelled_header]").exists()).toBe(true)
    })

    it("should show error for failed state", async () => {
      vi.mocked(getStatus).mockResolvedValueOnce({ status: "FAILED" })
      // twice needed because of "focus-hack"
      await vi.advanceTimersToNextTimerAsync()
      await vi.advanceTimersToNextTimerAsync()
      expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)
    })

    it("should show error for get status failure", async () => {
      vi.mocked(getStatus).mockRejectedValueOnce("failed" as ErrorType)
      // twice needed because of "focus-hack"
      await vi.advanceTimersToNextTimerAsync()
      await vi.advanceTimersToNextTimerAsync()
      expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)
    })

    it("should show error for get status network", async () => {
      vi.mocked(getStatus).mockRejectedValueOnce("network" as ErrorType)
      // twice needed because of "focus-hack"
      await vi.advanceTimersToNextTimerAsync()
      await vi.advanceTimersToNextTimerAsync()
      expect(wrapper.find("[data-testid=network_header]").exists()).toBe(true)
    })
  })

  it("should show error for post engagement failure", async () => {
    vi.mocked(createSession).mockRejectedValueOnce("failed" as ErrorType)

    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    await flushPromises()

    expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)
  })

  it("should show error for post engagement network", async () => {
    vi.mocked(createSession).mockRejectedValueOnce("network" as ErrorType)

    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    await flushPromises()

    expect(wrapper.find("[data-testid=network_header]").exists()).toBe(true)
  })

  it("should show qr code again after retrying for desktop mode", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    await flushPromises()

    expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)

    vi.mocked(getStatus).mockResolvedValueOnce({ status: "FAILED" })
    // twice needed because of "focus-hack"
    await vi.advanceTimersToNextTimerAsync()
    await vi.advanceTimersToNextTimerAsync()

    expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=qr]").exists()).toBe(false)

    await wrapper.find("[data-testid=retry_button]").trigger("click")

    await vi.waitFor(() => {
      expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
    })
  })

  it("should show device choice again after retrying for mobile mode", async () => {
    const wrapper = mount(WalletModal, {
      props: {
        startUrl: new URL("http://localhost/sessions"),
        usecase: "test123",
        helpBaseUrl: new URL("https://example.com"),
      },
      global: {
        provide: { [isMobileKey as symbol]: true, [translationsKey as symbol]: translations("nl") },
      },
    })
    await vi.advanceTimersToNextTimerAsync()

    vi.mocked(getStatus).mockResolvedValueOnce({ status: "FAILED" })
    expect(wrapper.find("[data-testid=device_choice]").exists()).toBe(true)
    await wrapper.find("[data-testid=same_device_button]").trigger("click")

    await vi.advanceTimersToNextTimerAsync()

    expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=device_choice]").exists()).toBe(false)

    await wrapper.find("[data-testid=retry_button]").trigger("click")

    expect(wrapper.find("[data-testid=device_choice]").exists()).toBe(true)
  })
})
