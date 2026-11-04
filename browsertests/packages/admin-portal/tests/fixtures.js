import { test as base, expect } from "@playwright/test"
import { AdminPortalPage } from "../pages/adminPortalPage.js"
import AxeBuilder from "@axe-core/playwright"

const DISABLED_RULES = [
  "select-name", // TODO =: enable after PVW-6463
  "scrollable-region-focusable", // TODO =: enable after PVW-6464
]

export const test = base.extend({
  adminPortal: async ({ page }, use) => {
    await use(new AdminPortalPage(page))
  },

  /** Checks the given page, by default the page of the test, against WCAG 2.1 A and AA. */
  accessibilityCheck: async ({ page }, use) => {
    async function check(target = page) {
      await target.waitForLoadState("load")
      const accessibilityScanResults = await new AxeBuilder({ page: target })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .disableRules(DISABLED_RULES)
        // The Vue DevTools button is only part of the local dev server, not of the portal.
        .exclude(".panel-entry-btn")
        .analyze()
      expect(accessibilityScanResults.violations).toEqual([])
    }
    await use(check)
  },
})
