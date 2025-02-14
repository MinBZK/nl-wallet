import { test, expect } from "@playwright/test"
const { GbaFetchPage } = require("../../pages/gbaFetchPage").default

test.describe("GBA Fetch when no client certificate is provided", () => {
  test("The webpage can only be accessed via VPN and using an approved user certificate.", async ({
    page,
    baseURL,
  }) => {
    await page.goto(baseURL)
    let gbaFetchPage = new GbaFetchPage(page)
    expect(await gbaFetchPage.getPageTitle()).toBe("400 Bad Request")
  })
})
