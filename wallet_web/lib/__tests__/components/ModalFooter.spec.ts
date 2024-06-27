import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"
import { isMobileKey } from "../../util/projection_keys"

await import("../setup")

describe("ModalFooter", () => {
  it("should render footer for close state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: FooterState.Close },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for close state for mobile", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: FooterState.Close },
      global: {
        provide: { [isMobileKey as symbol]: true },
      },
    })
    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for stop state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: FooterState.Stop },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for cancel state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: FooterState.Cancel },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for retry state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: FooterState.Retry },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(true)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for ok state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: FooterState.Ok },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })
})
