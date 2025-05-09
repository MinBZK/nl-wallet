import { test as base, expect } from "@playwright/test"
import { DemoPage } from "../pages/demoPage.js"
import AxeBuilder from "@axe-core/playwright"

const test = base.extend({
  demoPage: async ({ page, baseURL }, use) => {
    await page.goto(baseURL)
    await use(new DemoPage(page))
  },

  // eslint-disable-next-line no-empty-pattern -- fixtures require destructuring
  isMobileDevice: async ({}, use, testInfo) => {
    await use(testInfo.project.name.split("-")[1] === "mobile")
  },

  accessibilityCheck: async ({ page, demoPage }, use) => {
    async function check() {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include(demoPage.nlWalletButtonTag)
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .analyze()
      expect(accessibilityScanResults.violations).toEqual([])
    }
    await use(check)
  },
})

test.describe("UC 13.1 Verifier displays disclosure procedure on their front-end to User", () => {
  test("The library offers a standardized Start-button, the Verifier decides which button text to display.", async ({
    page,
    demoPage,
  }) => {
    await demoPage.goToAmsterdamMunicipality()
    expect(await demoPage.getWalletButtonText()).toContain("Login with NL Wallet")
    await page.goBack()
    await demoPage.goToXyzBank()
    expect(await demoPage.getWalletButtonText()).toBe("Use NL Wallet")
    await page.goBack()
    await demoPage.goToMarketplace()
    expect(await demoPage.getWalletButtonText()).toBe("Continue with NL Wallet")
    await page.goBack()
    await demoPage.goToMonkeyBike()
    expect(await demoPage.getWalletButtonText()).toBe("Continue with NL Wallet")
    await page.goBack()
  })

  test("When a user clicks one of these buttons, the library requests a new disclosure session from the Relying Party backend (to be implemented by the RP). The RP backend should request a new session from the OV and return the information to the library.", async ({
    page,
    demoPage,
    accessibilityCheck,
  }) => {
    const requestPromise = page.waitForRequest(
      (request) => request.url().includes("/disclosure/sessions/") && request.method() === "GET",
    )
    const responsePromise = page.waitForResponse(
      (response) => response.url().includes("/disclosure/sessions/") && response.status() === 200,
    )
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    const request = await requestPromise
    const response = await responsePromise
    const jsonResponse = await response.json()
    expect(request.url()).toContain("session_type")
    expect(jsonResponse).toHaveProperty("status")
    expect(jsonResponse).toHaveProperty("ul")
    await accessibilityCheck()
  })

  test("When the frontend library tries to fetch the session status, but this takes too long or fails, the user is warned that they may not have a good internet connection and offers to try again.", async ({
    context,
    page,
    demoPage,
    accessibilityCheck,
  }) => {
    await demoPage.goToAmsterdamMunicipality()
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.abort()
    })
    await context.setOffline(true)
    await demoPage.openWalletLogin()
    expect(await demoPage.getModalMessageHeaderText()).toBe("Sorry, no internet connection")
    expect(await demoPage.getModalMessageText()).toBe(
      "Your internet connection seems to be down or too slow. Check your connection and try again.",
    )
    expect(await demoPage.getWebsiteLink()).toBeDefined()
    await expect(await demoPage.getTryAgainButton()).toBeVisible()
    await expect(await demoPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test("When a mobile device is detected, and when the library cannot reliably detect that it runs on a desktop device, it asks the user where the NL Wallet is installed, offering options for same device flow, cross-device flow and to abort. When the library can reliably detect that it runs on a desktop device, it automatically starts the cross-device flow.", async ({
    demoPage,
    isMobileDevice,
    accessibilityCheck,
  }) => {
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    /* eslint-disable playwright/no-conditional-expect */
    // eslint-disable-next-line playwright/no-conditional-in-test,
    if (isMobileDevice) {
      await expect(await demoPage.getSameDeviceButton()).toBeVisible()
      await expect(await demoPage.getCrossDeviceButton()).toBeVisible()
      await expect(await demoPage.getQrCode()).toBeHidden()
      await expect(await demoPage.getCloseButton()).toBeVisible()
      expect(await demoPage.getWebsiteLink()).toBeDefined()
      await accessibilityCheck()
    } else {
      await expect(await demoPage.getSameDeviceButton()).toBeHidden()
      await expect(await demoPage.getCrossDeviceButton()).toBeHidden()
      await expect(await demoPage.getQrCode()).toBeVisible()
      await expect(await demoPage.getCloseButton()).toBeVisible()
      expect(await demoPage.getWebsiteLink()).toBeDefined()
      expect(await demoPage.getModalMessageHeaderText()).toBe("Scan the QR code with your NL Wallet app")
      await accessibilityCheck()
    }
    /* eslint-enable */
  })

  test("The QR is automatically refreshed every 2 second to prevent a passive attacker from just relaying the QR code to (potential) victims.", async ({
    page,
    demoPage,
  }) => {
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    await demoPage.startCrossDeviceFlow()
    const initialQrScreenshot = await demoPage.getQrScreenshot()
    await page.waitForTimeout(2100) // eslint-disable-line playwright/no-wait-for-timeout
    const newQrScreenshot = await demoPage.getQrScreenshot()
    expect(newQrScreenshot).not.toEqual(initialQrScreenshot)
  })

  test('The library polls the status of this session. Upon "failed", the library informs the user that something went wrong and offers the option to try again, which leads to a new session.', async ({
    page,
    demoPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 500,
      })
    })
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    expect(await demoPage.getModalMessageHeaderText()).toBe("Sorry, something went wrong")
    expect(await demoPage.getModalMessageText()).toBe(
      "This action was unsuccessful. This may have several reasons. Please try again.",
    )
    await expect(await demoPage.getTryAgainButton()).toBeVisible()
    expect(await demoPage.getWebsiteLink()).toBeDefined()
    await expect(await demoPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test('The library polls the status of this session. Upon "waiting for response", the library hides the QR code (when in cross-device) and tells the User to follow the instructions on their mobile device.', async ({
    page,
    demoPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "WAITING_FOR_RESPONSE" }),
      })
    })
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    expect(await demoPage.getModalMessageHeaderText()).toBe("Follow the steps in your NL Wallet app")
    await expect(await demoPage.getHelpLink()).toBeVisible()
    expect(await demoPage.getWebsiteLink()).toBeDefined()
    await expect(await demoPage.getCancelButton()).toBeVisible()
    await accessibilityCheck()
  })

  test('The library polls the status of this session. Upon "expired", the library informs the user that the session was expired and offers them the option to try again, which leads to a new session.', async ({
    page,
    demoPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "EXPIRED" }),
      })
    })
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    expect(await demoPage.getModalMessageHeaderText()).toBe("Sorry, time is over")
    expect(await demoPage.getModalMessageText()).toBe(
      "This action has been stopped because too much time has passed. This happens to keep your data safe. Please try again.",
    )
    await expect(await demoPage.getHelpLink()).toBeVisible()
    await expect(await demoPage.getTryAgainButton()).toBeVisible()
    await expect(await demoPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test('The library polls the status of this session. Upon "cancelled", the library confirms to the user that they have aborted the session and offers them the option to try again, which leads to a new session.', async ({
    page,
    demoPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "CANCELLED" }),
      })
    })
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    expect(await demoPage.getModalMessageHeaderText()).toBe("Stopped")
    expect(await demoPage.getModalMessageText()).toBe("Because you have stopped, no data has been shared.")
    await expect(await demoPage.getHelpLink()).toBeVisible()
    await expect(await demoPage.getTryAgainButton()).toBeVisible()
    await expect(await demoPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test("The library supports the following languages: Dutch, English. The language to be used is specified by the relying party.", async ({
    demoPage,
    isMobileDevice,
  }) => {
    await demoPage.setDutchLanguage()
    await demoPage.goToAmsterdamMunicipality()
    await demoPage.openWalletLogin()
    expect(await demoPage.getModalMessageHeaderText()).toBe(
      isMobileDevice ? "Op welk apparaat staat je NL Wallet app?" : "Scan de QR-code met je NL Wallet app",
    )
  })
})
