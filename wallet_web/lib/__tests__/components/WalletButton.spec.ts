import WalletButton from "@/components/WalletButton.ce.vue"
import { flushPromises, mount } from "@vue/test-utils"
import { describe, expect, it, vi } from "vitest"
import { nextTick } from "vue"

await import("../setup")

vi.mock("@/api/engagement")
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
})
