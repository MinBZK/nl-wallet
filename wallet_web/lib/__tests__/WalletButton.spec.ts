import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"
import WalletButton from "../WalletButton.ce.vue"

describe("WalletButton", () => {
  it("renders properly", () => {
    const wrapper = mount(WalletButton, { props: { text: "Hello Vitest" } })
    expect(wrapper.text()).toContain("Hello Vitest")
  })
})
