import { test as base, expect } from "@playwright/test"
import { PortalPage as PortalPage } from "../pages/portalPage.js"
import AxeBuilder from "@axe-core/playwright"

const test = base.extend({
  portalPage: async ({ page, baseURL }, use) => {
    await page.goto(`${baseURL}support/delete?lang=en`)
    await use(new PortalPage(page))
  },

  /* eslint-disable-next-line no-empty-pattern -- fixtures require destructuring */
  isMobileDevice: async ({}, use, testInfo) => {
    await use(testInfo.project.name.split("-")[1] === "mobile" || testInfo.project.name.split("-")[1] === "tablet")
  },

  accessibilityCheck: async ({ page }, use) => {
    async function check() {
      await page.waitForLoadState("load")
      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .analyze()
      expect(accessibilityScanResults.violations).toEqual([])
    }
    await use(check)
  },

  visualCheck: async ({ page }, use) => {
    await page.waitForLoadState("load")
    /* eslint-disable-next-line playwright/no-wait-for-timeout -- hard wait needed for screenshot test */
    await page.waitForTimeout(1000)
    const visualCheck = async (screenshotName) => {
      await expect(page).toHaveScreenshot(screenshotName, {
        maxDiffPixelRatio: 0.04,
      })
    }
    await use(visualCheck)
  },
})

test.describe("Revocation portal", () => {
  test("When a the revocation code does not exist an error message is displayed", async ({
    page,
    portalPage,
    accessibilityCheck,
    visualCheck,
  }) => {
    let requestData = null
    page.on("request", (request) => {
      if (request.method() === "POST" && request.url().includes("/support/delete")) {
        requestData = request.postData()
      }
    })

    await accessibilityCheck()
    await visualCheck("revocation-portal-page.png")

    await portalPage.enterRevocationCode("123456789012345678")
    await portalPage.submitRevocation()

    expect(requestData).toContain(`csrf_token=`)

    await accessibilityCheck()
    expect(await portalPage.getTitle()).toBe(`Sorry, something went wrong`)
    await visualCheck("revocation-portal-error.png")
  })

  test("It has multilanguage support", async ({ portalPage, accessibilityCheck, visualCheck }) => {
    await portalPage.setDutchLanguage()
    expect(await portalPage.getTitle()).toContain("Wil je je NL Wallet verwijderen?")
    await accessibilityCheck()
    await visualCheck("revocation-portal-dutch.png")

    await portalPage.enterRevocationCode("123456789012345678")
    await portalPage.submitRevocation()
    await portalPage.setEnglishLanguage()
    expect(await portalPage.getTitle()).toBe(`Sorry, something went wrong`)
    await visualCheck("revocation-portal-error-english.png")
  })

  test("It allows the user to cancel revocation", async ({ page, portalPage }) => {
    await portalPage.enterRevocationCode("123456789012345678")
    await portalPage.cancelRevocation()

    await expect(page).toHaveURL(/\/support\/$/)
  })

  test("It validates CSRF token on form submission", async ({ page, portalPage, visualCheck }) => {
    await portalPage.enterRevocationCode("123456789012345678")

    await page.locator('input[name="csrf_token"]').inputValue()

    await page.route("**/support/delete", async (route) => {
      const request = route.request()
      if (request.method() === "POST") {
        const postData = await request.postData()
        const modifiedData = postData.replace(/csrf_token=[^&]+/, "csrf_token=invalid_token_123")

        await route.continue({
          postData: modifiedData,
        })
      } else {
        await route.continue()
      }
      await portalPage.submitRevocation()
      const response = await page.waitForResponse(
        (resp) => resp.url().includes("/support/delete") && resp.request().method() === "POST",
      )
      expect(response.status()).toBe(303)
    })
    await visualCheck("revocation-portal-csrf-error.png")
  })

  test("It can be used when javascript is disabled", async ({ browser }) => {
    const context = await browser.newContext({
      javaScriptEnabled: false,
    })
    const page = await context.newPage()
    await page.goto(`support/delete?lang=en`)
    const portalPage = new PortalPage(page)

    await portalPage.enterRevocationCode("123456789012345678")
    await portalPage.submitRevocation()

    expect(await portalPage.getTitle()).toBe(`Sorry, something went wrong`)
  })
})
