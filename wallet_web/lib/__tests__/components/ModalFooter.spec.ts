import ModalFooter from "@/components/ModalFooter.vue"
import { SessionState } from "@/models/state"
import { SessionType } from "@/models/status"
import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"

await import("../setup")

describe("ModalFooter", () => {
  it("should render footer for created state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: SessionState.Created, type: null },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for loading state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: SessionState.Loading, type: null },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for in-progress state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: SessionState.InProgress, type: null },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for retry state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: SessionState.Error, type: null },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(true)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for success state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: SessionState.Success, type: SessionType.CrossDevice },
    })
    expect(wrapper.find("[data-testid=close_button].link").exists()).toBe(true)
    expect(wrapper.find("[data-testid=close_button].primary").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for success state with same device", async () => {
    const wrapper = mount(ModalFooter, {
      props: { state: SessionState.Success, type: SessionType.SameDevice },
    })
    expect(wrapper.find("[data-testid=close_button].link").exists()).toBe(false)
    expect(wrapper.find("[data-testid=close_button].primary").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })
})
