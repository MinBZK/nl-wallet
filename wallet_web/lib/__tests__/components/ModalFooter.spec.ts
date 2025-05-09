import ModalFooter from "@/components/ModalFooter.vue"
import { translations, translationsKey } from "@/util/translations"
import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"

await import("../setup")

describe("ModalFooter", () => {
  it("should render footer for loading state", async () => {
    const wrapper = mount(ModalFooter, {
      props: {
        modalState: { kind: "creating" },
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for created state", async () => {
    const wrapper = mount(ModalFooter, {
      props: {
        modalState: {
          kind: "created",
          sameDeviceUl: new URL("example://app.example.com/-/?session_type=same_device"),
          crossDeviceUl: new URL("example://app.example.com/-/?session_type=cross_device"),
          session: {
            statusUrl: new URL("http://status.example.com/status"),
            sessionType: "cross_device",
            sessionToken: "123",
          },
        },
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for loading state", async () => {
    const wrapper = mount(ModalFooter, {
      props: {
        modalState: {
          kind: "loading",
          session: {
            statusUrl: new URL("http://status.example.com/status"),
            sessionType: "cross_device",
            sessionToken: "123",
          },
        },
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for in-progress state", async () => {
    const wrapper = mount(ModalFooter, {
      props: {
        modalState: {
          kind: "in-progress",
          session: {
            statusUrl: new URL("http://status.example.com/status"),
            sessionType: "cross_device",
            sessionToken: "123",
          },
        },
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(true)
  })

  it("should render footer for retry state", async () => {
    const wrapper = mount(ModalFooter, {
      props: { modalState: { kind: "error", errorType: "failed" } },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(true)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for success state", async () => {
    const wrapper = mount(ModalFooter, {
      props: {
        modalState: {
          kind: "success",
          session: {
            statusUrl: new URL("http://status.example.com/status"),
            sessionType: "cross_device",
            sessionToken: "123",
          },
        },
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button].link").exists()).toBe(true)
    expect(wrapper.find("[data-testid=close_button].primary").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })

  it("should render footer for success state with same device", async () => {
    const wrapper = mount(ModalFooter, {
      props: {
        modalState: {
          kind: "success",
          session: {
            statusUrl: new URL("http://status.example.com/status"),
            sessionType: "same_device",
            sessionToken: "123",
          },
        },
      },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=close_button].link").exists()).toBe(false)
    expect(wrapper.find("[data-testid=close_button].primary").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancel_button]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=retry_button]").exists()).toBe(false)

    expect(wrapper.find("[data-testid=help]").exists()).toBe(false)
  })
})
