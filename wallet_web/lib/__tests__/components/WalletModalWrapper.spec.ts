import WalletModal from "@/components/WalletModal.vue"
import DynamicWalletModal from "@/components/DynamicWalletModal.vue"
import StaticWalletModal from "@/components/StaticWalletModal.vue"

import { getStatus } from "@/api/status"
import { translations, translationsKey } from "@/util/translations"
import { isMobileKey } from "@/util/useragent"
import { flushPromises, mount } from "@vue/test-utils"
import { beforeEach, describe, expect, it, vi } from "vitest"

await import("../setup")

vi.mock("@/api/cancel")
vi.mock("@/api/session")
vi.mock("@/api/status")

describe("WalletModal", () => {
  describe("dynamic strategy", () => {
    beforeEach(() => {
      vi.useFakeTimers()
      vi.clearAllTimers()
    })

    it("should render DynamicWalletModal when strategy is dynamic", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "dynamic",
            usecase: "test123",
            startUrl: new URL("http://localhost/sessions"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: false,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })
      await flushPromises()

      expect(wrapper.findComponent(DynamicWalletModal).exists()).toBe(true)
      expect(wrapper.findComponent(StaticWalletModal).exists()).toBe(false)
    })

    it("should emit success event from DynamicWalletModal", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "dynamic",
            usecase: "test123",
            startUrl: new URL("http://localhost/sessions"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: false,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })

      await vi.mocked(getStatus).withImplementation(
        async () => ({
          status: "DONE",
          ul: new URL("example://app.example.com/-/"),
        }),
        async () => {
          await vi.advanceTimersToNextTimerAsync()
          await vi.advanceTimersToNextTimerAsync()

          await wrapper.find("[data-testid=close_button]").trigger("click")

          expect(wrapper.emitted("success")).toBeTruthy()
          expect(wrapper.emitted("success")![0]).toEqual(["mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl", "cross_device"])
        },
      )
    })

    it("should emit failed event from DynamicWalletModal", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "dynamic",
            usecase: "test123",
            startUrl: new URL("http://localhost/sessions"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: { provide: { [translationsKey as symbol]: translations("nl") } },
      })
      await flushPromises()

      vi.mocked(getStatus).mockResolvedValueOnce({ status: "FAILED" })
      await vi.advanceTimersToNextTimerAsync()
      await vi.advanceTimersToNextTimerAsync()

      expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)

      await wrapper.find("[data-testid=close_button]").trigger("click")

      expect(wrapper.emitted("failed")).toBeTruthy()
      expect(wrapper.emitted("failed")![0]).toEqual(["mkwL0sHfP2cLJcRMuDzCHXEofujk9nnl", "cross_device"])
    })
  })

  describe("static strategy", () => {
    it("should render StaticWalletModal when strategy is static", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "static",
            sameDeviceUl: new URL("example://app.example.com/same"),
            crossDeviceUl: new URL("example://app.example.com/cross"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: false,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })
      await flushPromises()

      expect(wrapper.findComponent(StaticWalletModal).exists()).toBe(true)
      expect(wrapper.findComponent(DynamicWalletModal).exists()).toBe(false)
    })

    it("should emit close event from StaticWalletModal", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "static",
            sameDeviceUl: new URL("example://app.example.com/same"),
            crossDeviceUl: new URL("example://app.example.com/cross"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: false,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })
      await flushPromises()

      const staticModal = wrapper.findComponent(StaticWalletModal)
      staticModal.vm.$emit("close")

      expect(wrapper.emitted("close")).toBeTruthy()
    })

    it("should render device choice on mobile", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "static",
            sameDeviceUl: new URL("example://app.example.com/same"),
            crossDeviceUl: new URL("example://app.example.com/cross"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: true,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })
      await flushPromises()

      expect(wrapper.find("[data-testid=device_choice]").exists()).toBe(true)
    })

    it("should render QR code on desktop", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "static",
            sameDeviceUl: new URL("example://app.example.com/same"),
            crossDeviceUl: new URL("example://app.example.com/cross"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: false,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })
      await flushPromises()

      expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
    })

    it("should emit close when close button is clicked", async () => {
      const wrapper = mount(WalletModal, {
        props: {
          modalType: {
            strategy: "static",
            sameDeviceUl: new URL("example://app.example.com/same"),
            crossDeviceUl: new URL("example://app.example.com/cross"),
          },
          helpBaseUrl: new URL("https://example.com"),
        },
        global: {
          provide: {
            [isMobileKey as symbol]: true,
            [translationsKey as symbol]: translations("nl"),
          },
        },
      })
      await flushPromises()

      await wrapper.find("[data-testid=close_button]").trigger("click")

      expect(wrapper.emitted("close")).toBeTruthy()
    })
  })
})
