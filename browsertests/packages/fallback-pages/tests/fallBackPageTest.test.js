import { test, expect } from "@playwright/test"
import { FallbackPage } from "../pages/fallbackPage.js"

test.describe("Universal link fallback page", () => {
  test("When wallet app is not installed opening a universal link will open the fallback page", async ({
    page,
    baseURL,
  }) => {
    let fallbackPage = new FallbackPage(page)
    await page.goto(baseURL + "dummy-deep-link")
    expect(await fallbackPage.getPageTitle()).toBe("Download de NL Wallet app")
    await expect(fallbackPage.getDeeplink()).toBeVisible()
    await expect(fallbackPage.getHelpLink()).toBeVisible()
    await expect(fallbackPage.getStoreBanners()).toBeVisible()
    await page.goto(baseURL + "dummy-deep-link?retry=true")
    expect(await fallbackPage.getPageTitle()).toBe("Download de NL Wallet app")
    await expect(fallbackPage.getDeeplink()).not.toBeVisible()
  })
})
