import DeviceChoice from "@/components/DeviceChoice.vue"
import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"

await import("../setup")

describe("DeviceChoice", () => {
  it("should show same device link with engagement url", async () => {
    const wrapper = mount(DeviceChoice, {
      props: { engagementUrl: "engagement_url_123" },
    })
    const button = wrapper.find("[data-testid=same_device_button]")
    expect(button.exists()).toBe(true)
    expect(button.attributes("href")).toEqual("engagement_url_123")
  })

  it("should emit close", async () => {
    const wrapper = mount(DeviceChoice, {
      props: { engagementUrl: "engagement_url_123" },
    })
    await wrapper.find("[data-testid=close_button]").trigger("click")
    expect(wrapper.emitted()).toHaveProperty("close")
  })

  it("should show loading indicator when clicking same device", async () => {
    const wrapper = mount(DeviceChoice, {
      props: { engagementUrl: "engagement_url_123" },
    })

    const button = wrapper.find("[data-testid=same_device_button]")
    await button.trigger("click")

    expect(wrapper.find("[data-testid=same_device_loading_indicator]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cross_device_loading_indicator]").exists()).toBe(true)
    expect(button.attributes("class")).toContain("disabled")
    expect(
      wrapper.find("[data-testid=cross_device_button]").attributes("disabled"),
    ).not.toBeUndefined()
  })

  it("should show loading indicator when clicking cross device", async () => {
    const wrapper = mount(DeviceChoice, {
      props: { engagementUrl: "engagement_url_123" },
    })

    const button = wrapper.find("[data-testid=cross_device_button]")
    await button.trigger("click")

    expect(wrapper.find("[data-testid=same_device_loading_indicator]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cross_device_loading_indicator]").exists()).toBe(true)
    expect(button.attributes("disabled")).not.toBeUndefined()
    expect(wrapper.find("[data-testid=same_device_button]").attributes("class")).toContain(
      "disabled",
    )
  })
})
