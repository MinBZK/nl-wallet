import { expect } from "@playwright/test"
import { test } from "../common.js"

test.describe("GBA Fetch when no client certificate is provided", () => {
  test("The webpage can only be accessed via VPN and using an approved user certificate.", async ({
    gbaFetchPage,
  }) => {
    expect(await gbaFetchPage.getPageTitle()).toBe("400 Bad Request")
  })
})
