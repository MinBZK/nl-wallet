import ErrorSection from "@/components/ErrorSection.vue"
import { translations, translationsKey } from "@/util/translations"
import { mount } from "@vue/test-utils"
import { describe, expect, it } from "vitest"

await import("../setup")

describe("ErrorSection", () => {
  it("should render error for failed status", async () => {
    const wrapper = mount(ErrorSection, {
      props: { errorType: "failed", helpBaseUrl: new URL("https://example.com") },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=expired_header]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=cancelled_header]").exists()).toBe(false)
  })

  it("should render error for cancelled status", async () => {
    const wrapper = mount(ErrorSection, {
      props: { errorType: "cancelled", helpBaseUrl: new URL("https://example.com") },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=expired_header]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancelled_header]").exists()).toBe(true)
  })

  it("should render error for expired status", async () => {
    const wrapper = mount(ErrorSection, {
      props: { errorType: "expired", helpBaseUrl: new URL("https://example.com") },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    expect(wrapper.find("[data-testid=expired_header]").exists()).toBe(true)
    expect(wrapper.find("[data-testid=failed_header]").exists()).toBe(false)
    expect(wrapper.find("[data-testid=cancelled_header]").exists()).toBe(false)
  })
})
