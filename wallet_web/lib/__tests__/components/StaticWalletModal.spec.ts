import StaticWalletModal from "@/components/StaticWalletModal.vue"
import ModalMain from "@/components/ModalMain.vue"

import { translations, translationsKey } from "@/util/translations"
import { isMobileKey } from "@/util/useragent"
import { flushPromises, mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"

await import("../setup")

describe("StaticWalletModal", () => {
  describe("static URL handling", () => {
    it("should have correct same_device link", async () => {
      const wrapper = mount(StaticWalletModal, {
        props: {
          sameDeviceUl: new URL("example://app.example.com/same"),
          crossDeviceUl: new URL("example://app.example.com/cross"),
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

      const sameDeviceButton = wrapper.find("[data-testid=same_device_button]")
      expect(sameDeviceButton.attributes("href")).toEqual("example://app.example.com/same")
    })
  })

  describe("handleChoice", () => {
    it("should update session type when choosing cross_device from created state", async () => {
      const wrapper = mount(StaticWalletModal, {
        props: {
          sameDeviceUl: new URL("example://app.example.com/same"),
          crossDeviceUl: new URL("example://app.example.com/cross"),
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

      await wrapper.find("[data-testid=cross_device_button]").trigger("click")

      expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)
    })

    it("should keep created state and update session type on multiple choice calls", async () => {
      const wrapper = mount(StaticWalletModal, {
        props: {
          sameDeviceUl: new URL("example://app.example.com/same"),
          crossDeviceUl: new URL("example://app.example.com/cross"),
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

      // First choice - cross_device
      await wrapper.find("[data-testid=cross_device_button]").trigger("click")
      await flushPromises()

      // Should show QR code now
      expect(wrapper.find("[data-testid=qr]").exists()).toBe(true)

      // Emit another choice - component stays in created state
      const main = wrapper.findComponent(ModalMain)
      main.vm.$emit("choice", "same_device")
      await flushPromises()

      // Should still work (state remains "created")
      expect(wrapper.find("[data-testid=wallet_modal]").exists()).toBe(true)
    })

    it("should set error state when choice is triggered from non-created state", async () => {
      const wrapper = mount(StaticWalletModal, {
        props: {
          sameDeviceUl: new URL("example://app.example.com/same"),
          crossDeviceUl: new URL("example://app.example.com/cross"),
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

      const vm = wrapper.vm as any
      vm.modalState = {
        kind: "success",
        session: {
          statusUrl: new URL("http://status.example.com/status"),
          sessionType: "same_device",
          sessionToken: "test-token",
        },
      }
      await flushPromises()

      const main = wrapper.findComponent(ModalMain)
      main.vm.$emit("choice", "cross_device")
      await flushPromises()

      expect(vm.modalState.kind).toBe("error")
      expect(vm.modalState.errorType).toBe("failed")
    })

    it("should set error state without session when choice is triggered from creating state", async () => {
      const wrapper = mount(StaticWalletModal, {
        props: {
          sameDeviceUl: new URL("example://app.example.com/same"),
          crossDeviceUl: new URL("example://app.example.com/cross"),
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

      const vm = wrapper.vm as any
      vm.modalState = {
        kind: "creating",
      }
      await flushPromises()

      const main = wrapper.findComponent(ModalMain)
      main.vm.$emit("choice", "cross_device")
      await flushPromises()

      expect(vm.modalState.kind).toBe("error")
      expect(vm.modalState.errorType).toBe("failed")
      expect(vm.modalState.session).toBeUndefined()
    })
  })
})
