import { test as base, expect } from "@playwright/test"
import { FallbackPage } from "../pages/fallbackPage.js"

export const test = base.extend({
  fallbackPage: async ({ page }, use) => {
    await use(new FallbackPage(page))
  },
})

test.describe("Universal link fallback page", () => {
  test("When wallet app is not installed opening a universal link will open the fallback page", async ({
    page,
    baseURL,
    fallbackPage,
  }) => {
    await page.goto(baseURL + "dummy-deep-link")
    expect(await fallbackPage.getPageTitle()).toBe("Download de NL Wallet app")
    await expect(fallbackPage.getDeeplink()).toBeVisible()
    await expect(fallbackPage.getHelpLink()).toBeVisible()
    await expect(fallbackPage.getStoreBanners()).toBeVisible()
    await page.goto(baseURL + "dummy-deep-link?retry=true")
    expect(await fallbackPage.getPageTitle()).toBe("Download de NL Wallet app")
    await expect(fallbackPage.getDeeplink()).toBeHidden()
  })
})
