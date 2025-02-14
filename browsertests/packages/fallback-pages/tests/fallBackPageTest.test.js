import { test, expect } from "@playwright/test"
const { FallbackPage } = require("../pages/fallbackPage").default

test.describe("Universal link fallback page", () => {
  test("When wallet app is not installed opening a universal link will open the fallback page", async ({
    page,
    baseURL,
  }) => {
    let fallbackPage = new FallbackPage(page)
    await page.goto(baseURL + "dummy-deep-link")
    expect(await fallbackPage.getPageTitle()).toBe("Download de NL Wallet app")
    expect(await fallbackPage.getDeeplink()).toBeVisible()
    expect(await fallbackPage.getHelplink()).toBeVisible()
    expect(await fallbackPage.getStoreBanners()).toBeVisible()
    await page.goto(baseURL + "dummy-deep-link?retry=true")
    expect(await fallbackPage.getPageTitle()).toBe("Download de NL Wallet app")
    expect(await fallbackPage.getDeeplink()).not.toBeVisible()
  })
})
