import { test as base } from "@playwright/test"
import { GbaFetchPage } from "../pages/gbaFetchPage.js"

export const test = base.extend({
  gbaFetchPage: async ({ page, baseURL }, use) => {
    await page.goto(baseURL)
    await use(new GbaFetchPage(page))
  },
})
