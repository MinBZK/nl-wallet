import WalletButton from "@/components/WalletButton.ce.vue"
import WalletModal from "@/components/WalletModal.vue"
import { flushPromises, mount } from "@vue/test-utils"
import { describe, expect, it, vi } from "vitest"
import { nextTick } from "vue"

await import("../setup")

vi.mock("@/api/session")
vi.mock("@/api/status")

describe("WalletButton", () => {
  it("shows the provided text in buttons", () => {
    const wrapper = mount(WalletButton, {
      props: { text: "Wallet button test 123", usecase: "test123" },
    })
    expect(wrapper.text()).toContain("Wallet button test 123")
  })

  it("should open modal", async () => {
    const wrapper = mount(WalletButton, { props: { text: "inloggen", usecase: "test123" } })
    await wrapper.find("[data-testid=wallet_button]").trigger("click")
    await nextTick()
    await flushPromises()

    expect(wrapper.text()).toContain("NL Wallet")
  })

  it("should emit success event and close modal when WalletModal emits success", async () => {
    const wrapper = mount(WalletButton, { props: { text: "inloggen", usecase: "test123" } })
    await wrapper.find("[data-testid=wallet_button]").trigger("click")
    await nextTick()
    await flushPromises()

    const modal = wrapper.findComponent(WalletModal)
    expect(modal.exists()).toBe(true)

    modal.vm.$emit("success", "token123", "cross_device")
    await nextTick()

    expect(wrapper.emitted("success")).toBeTruthy()
    expect(wrapper.emitted("success")![0]).toEqual(["token123", "cross_device"])
    expect(wrapper.findComponent(WalletModal).exists()).toBe(false)
  })

  it("should emit failed event and close modal when WalletModal emits failed", async () => {
    const wrapper = mount(WalletButton, { props: { text: "inloggen", usecase: "test123" } })
    await wrapper.find("[data-testid=wallet_button]").trigger("click")
    await nextTick()
    await flushPromises()

    const modal = wrapper.findComponent(WalletModal)
    expect(modal.exists()).toBe(true)

    modal.vm.$emit("failed", "token456", "same_device")
    await nextTick()

    expect(wrapper.emitted("failed")).toBeTruthy()
    expect(wrapper.emitted("failed")![0]).toEqual(["token456", "same_device"])
    expect(wrapper.findComponent(WalletModal).exists()).toBe(false)
  })

  it("should close modal when WalletModal emits close", async () => {
    const wrapper = mount(WalletButton, { props: { text: "inloggen", usecase: "test123" } })
    await wrapper.find("[data-testid=wallet_button]").trigger("click")
    await nextTick()
    await flushPromises()

    const modal = wrapper.findComponent(WalletModal)
    expect(modal.exists()).toBe(true)

    modal.vm.$emit("close")
    await nextTick()

    expect(wrapper.findComponent(WalletModal).exists()).toBe(false)
  })

  it("should use static strategy when sameDeviceUl and crossDeviceUl are provided", async () => {
    const wrapper = mount(WalletButton, {
      props: {
        text: "inloggen",
        sameDeviceUl: new URL("example://app.example.com/same"),
        crossDeviceUl: new URL("example://app.example.com/cross"),
      },
    })
    await wrapper.find("[data-testid=wallet_button]").trigger("click")
    await nextTick()
    await flushPromises()

    expect(wrapper.text()).toContain("NL Wallet")
  })
})
