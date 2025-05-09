import { test as base, expect } from "@playwright/test"
import { DemoRelyingPartyPage } from "../pages/demoRelyingPartyPage.js"
import AxeBuilder from "@axe-core/playwright"

const test = base.extend({
  demoRelyingPartyPage: async ({ page, baseURL }, use) => {
    await page.goto(baseURL)
    await use(new DemoRelyingPartyPage(page))
  },

  // eslint-disable-next-line no-empty-pattern -- fixtures require destructuring
  isMobileDevice: async ({}, use, testInfo) => {
    await use(testInfo.project.name.split("-")[1] === "mobile")
  },

  accessibilityCheck: async ({ page, demoRelyingPartyPage }, use) => {
    async function check() {
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include(demoRelyingPartyPage.nlWalletButtonTag)
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
    demoRelyingPartyPage,
  }) => {
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    expect(await demoRelyingPartyPage.getWalletButtonText()).toContain("Login with NL Wallet")
    await page.goBack()
    await demoRelyingPartyPage.goToXyzBank()
    expect(await demoRelyingPartyPage.getWalletButtonText()).toBe("Use NL Wallet")
    await page.goBack()
    await demoRelyingPartyPage.goToMarketplace()
    expect(await demoRelyingPartyPage.getWalletButtonText()).toBe("Continue with NL Wallet")
    await page.goBack()
    await demoRelyingPartyPage.goToMonkeyBike()
    expect(await demoRelyingPartyPage.getWalletButtonText()).toBe("Continue with NL Wallet")
    await page.goBack()
  })

  test("When a user clicks one of these buttons, the library requests a new disclosure session from the Relying Party backend (to be implemented by the RP). The RP backend should request a new session from the OV and return the information to the library.", async ({
    page,
    demoRelyingPartyPage,
    accessibilityCheck,
  }) => {
    const requestPromise = page.waitForRequest(
      (request) => request.url().includes("/disclosure/sessions/") && request.method() === "GET",
    )
    const responsePromise = page.waitForResponse(
      (response) => response.url().includes("/disclosure/sessions/") && response.status() === 200,
    )
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
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
    demoRelyingPartyPage,
    accessibilityCheck,
  }) => {
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.abort()
    })
    await context.setOffline(true)
    await demoRelyingPartyPage.openWalletLogin()
    expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe("Sorry, no internet connection")
    expect(await demoRelyingPartyPage.getModalMessageText()).toBe(
      "Your internet connection seems to be down or too slow. Check your connection and try again.",
    )
    expect(await demoRelyingPartyPage.getWebsiteLink()).toBeDefined()
    await expect(await demoRelyingPartyPage.getTryAgainButton()).toBeVisible()
    await expect(await demoRelyingPartyPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test("When a mobile device is detected, and when the library cannot reliably detect that it runs on a desktop device, it asks the user where the NL Wallet is installed, offering options for same device flow, cross-device flow and to abort. When the library can reliably detect that it runs on a desktop device, it automatically starts the cross-device flow.", async ({
    demoRelyingPartyPage,
    isMobileDevice,
    accessibilityCheck,
  }) => {
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    /* eslint-disable playwright/no-conditional-expect */
    // eslint-disable-next-line playwright/no-conditional-in-test,
    if (isMobileDevice) {
      await expect(await demoRelyingPartyPage.getSameDeviceButton()).toBeVisible()
      await expect(await demoRelyingPartyPage.getCrossDeviceButton()).toBeVisible()
      await expect(await demoRelyingPartyPage.getQrCode()).toBeHidden()
      await expect(await demoRelyingPartyPage.getCloseButton()).toBeVisible()
      expect(await demoRelyingPartyPage.getWebsiteLink()).toBeDefined()
      await accessibilityCheck()
    } else {
      await expect(await demoRelyingPartyPage.getSameDeviceButton()).toBeHidden()
      await expect(await demoRelyingPartyPage.getCrossDeviceButton()).toBeHidden()
      await expect(await demoRelyingPartyPage.getQrCode()).toBeVisible()
      await expect(await demoRelyingPartyPage.getCloseButton()).toBeVisible()
      expect(await demoRelyingPartyPage.getWebsiteLink()).toBeDefined()
      expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe("Scan the QR code with your NL Wallet app")
      await accessibilityCheck()
    }
    /* eslint-enable */
  })

  test("The QR is automatically refreshed every 2 second to prevent a passive attacker from just relaying the QR code to (potential) victims.", async ({
    page,
    demoRelyingPartyPage,
  }) => {
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    await demoRelyingPartyPage.startCrossDeviceFlow()
    const initialQrScreenshot = await demoRelyingPartyPage.getQrScreenshot()
    await page.waitForTimeout(2100) // eslint-disable-line playwright/no-wait-for-timeout
    const newQrScreenshot = await demoRelyingPartyPage.getQrScreenshot()
    expect(newQrScreenshot).not.toEqual(initialQrScreenshot)
  })

  test('The library polls the status of this session. Upon "failed", the library informs the user that something went wrong and offers the option to try again, which leads to a new session.', async ({
    page,
    demoRelyingPartyPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 500,
      })
    })
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe("Sorry, something went wrong")
    expect(await demoRelyingPartyPage.getModalMessageText()).toBe(
      "This action was unsuccessful. This may have several reasons. Please try again.",
    )
    await expect(await demoRelyingPartyPage.getTryAgainButton()).toBeVisible()
    expect(await demoRelyingPartyPage.getWebsiteLink()).toBeDefined()
    await expect(await demoRelyingPartyPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test('The library polls the status of this session. Upon "waiting for response", the library hides the QR code (when in cross-device) and tells the User to follow the instructions on their mobile device.', async ({
    page,
    demoRelyingPartyPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "WAITING_FOR_RESPONSE" }),
      })
    })
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe("Follow the steps in your NL Wallet app")
    await expect(await demoRelyingPartyPage.getHelpLink()).toBeVisible()
    expect(await demoRelyingPartyPage.getWebsiteLink()).toBeDefined()
    await expect(await demoRelyingPartyPage.getCancelButton()).toBeVisible()
    await accessibilityCheck()
  })

  test('The library polls the status of this session. Upon "expired", the library informs the user that the session was expired and offers them the option to try again, which leads to a new session.', async ({
    page,
    demoRelyingPartyPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "EXPIRED" }),
      })
    })
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe("Sorry, time is over")
    expect(await demoRelyingPartyPage.getModalMessageText()).toBe(
      "This action has been stopped because too much time has passed. This happens to keep your data safe. Please try again.",
    )
    await expect(await demoRelyingPartyPage.getHelpLink()).toBeVisible()
    await expect(await demoRelyingPartyPage.getTryAgainButton()).toBeVisible()
    await expect(await demoRelyingPartyPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test('The library polls the status of this session. Upon "cancelled", the library confirms to the user that they have aborted the session and offers them the option to try again, which leads to a new session.', async ({
    page,
    demoRelyingPartyPage,
    accessibilityCheck,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "CANCELLED" }),
      })
    })
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe("Stopped")
    expect(await demoRelyingPartyPage.getModalMessageText()).toBe("Because you have stopped, no data has been shared.")
    await expect(await demoRelyingPartyPage.getHelpLink()).toBeVisible()
    await expect(await demoRelyingPartyPage.getTryAgainButton()).toBeVisible()
    await expect(await demoRelyingPartyPage.getCloseButton()).toBeVisible()
    await accessibilityCheck()
  })

  test("The library supports the following languages: Dutch, English. The language to be used is specified by the relying party.", async ({
    demoRelyingPartyPage,
    isMobileDevice,
  }) => {
    await demoRelyingPartyPage.setDutchLanguage()
    await demoRelyingPartyPage.goToAmsterdamMunicipality()
    await demoRelyingPartyPage.openWalletLogin()
    expect(await demoRelyingPartyPage.getModalMessageHeaderText()).toBe(
      isMobileDevice ? "Op welk apparaat staat je NL Wallet app?" : "Scan de QR-code met je NL Wallet app",
    )
  })
})
