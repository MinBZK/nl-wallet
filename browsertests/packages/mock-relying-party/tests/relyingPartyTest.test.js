import { test, expect } from "@playwright/test"
import { MockRelyingPartyPage } from "../pages/mockRelyingPartyPage.js"

test.describe("UC 13.1 Verifier displays disclosure procedure on their front-end to User", () => {
  let mockRelyingPartyPage
  test.beforeEach(async ({ page, baseURL }) => {
    mockRelyingPartyPage = new MockRelyingPartyPage(page)
    await page.goto(baseURL)
  })

  test("The library offers a standardized Start-button, the Verifier decides which button text to display.", async ({
    page,
  }) => {
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    expect(await mockRelyingPartyPage.getWalletButtonText()).toContain("Login with NL Wallet")
    await page.goBack()
    await mockRelyingPartyPage.goToXyzBank()
    expect(await mockRelyingPartyPage.getWalletButtonText()).toBe("Use NL Wallet")
    await page.goBack()
    await mockRelyingPartyPage.goToMarketplace()
    expect(await mockRelyingPartyPage.getWalletButtonText()).toBe("Continue with NL Wallet")
    await page.goBack()
    await mockRelyingPartyPage.goToMonkeyBike()
    expect(await mockRelyingPartyPage.getWalletButtonText()).toBe("Continue with NL Wallet")
    await page.goBack()
  })

  test("When a user clicks one of these buttons, the library requests a new disclosure session from the Relying Party backend (to be implemented by the RP). The RP backend should request a new session from the OV and return the information to the library.", async ({
    page,
  }) => {
    const requestPromise = page.waitForRequest(
      (request) => request.url().includes("/disclosure/sessions/") && request.method() === "GET",
    )
    const responsePromise = page.waitForResponse(
      (response) => response.url().includes("/disclosure/sessions/") && response.status() === 200,
    )
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    const request = await requestPromise
    const response = await responsePromise
    const jsonResponse = await response.json()
    expect(request.url()).toContain("session_type")
    expect(jsonResponse).toHaveProperty("status")
    expect(jsonResponse).toHaveProperty("ul")
  })

  test("When the frontend library tries to fetch the session status, but this takes too long or fails, the user is warned that they may not have a good internet connection and offers to try again.", async ({
    context,
    page,
  }) => {
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.abort()
    })
    await context.setOffline(true)
    await mockRelyingPartyPage.openWalletLogin()
    expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe(
      "Sorry, no internet connection",
    )
    expect(await mockRelyingPartyPage.getModalMessageText()).toBe(
      "Your internet connection seems to be down or too slow. Check your connection and try again.",
    )
    expect(await mockRelyingPartyPage.getWebsiteLink()).toBeDefined()
    await expect(await mockRelyingPartyPage.getTryAgainButton()).toBeVisible()
    await expect(await mockRelyingPartyPage.getCloseButton()).toBeVisible()
  })

  test("When a mobile device is detected, and when the library cannot reliably detect that it runs on a desktop device, it asks the user where the NL Wallet is installed, offering options for same device flow, cross-device flow and to abort. When the library can reliably detect that it runs on a desktop device, it automatically starts the cross-device flow.", async ({
    page,
  }, testInfo) => {
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    const screenSizeName = testInfo.project.name.split("-")[1]
    if (screenSizeName === "Mobile") {
      await expect(await mockRelyingPartyPage.getSameDeviceButton()).toBeVisible()
      await expect(await mockRelyingPartyPage.getCrossDeviceButton()).toBeVisible()
      await expect(await mockRelyingPartyPage.getQrCode()).not.toBeVisible()
      await expect(await mockRelyingPartyPage.getCloseButton()).toBeVisible()
      expect(await mockRelyingPartyPage.getWebsiteLink()).toBeDefined()
    } else {
      await expect(await mockRelyingPartyPage.getSameDeviceButton()).not.toBeVisible()
      await expect(await mockRelyingPartyPage.getCrossDeviceButton()).not.toBeVisible()
      await expect(await mockRelyingPartyPage.getQrCode()).toBeVisible()
      await expect(await mockRelyingPartyPage.getCloseButton()).toBeVisible()
      expect(await mockRelyingPartyPage.getWebsiteLink()).toBeDefined()
      expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe(
        "Scan the QR code with your NL Wallet app",
      )
    }
  })

  test("The QR is automatically refreshed every 2 second to prevent a passive attacker from just relaying the QR code to (potential) victims.", async ({
    page,
  }, testInfo) => {
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    const screenSizeName = testInfo.project.name.split("-")[1]
    if (screenSizeName === "Mobile") {
      await expect(await mockRelyingPartyPage.getSameDeviceButton()).toBeVisible()
      await expect(await mockRelyingPartyPage.getCrossDeviceButton()).toBeVisible()
      await mockRelyingPartyPage.startCrossDeviceFlow()
    }
    const initialQrScreenshot = await mockRelyingPartyPage.getQrScreenshot()
    await page.waitForTimeout(2100)
    const newQrScreenshot = await mockRelyingPartyPage.getQrScreenshot()
    expect(newQrScreenshot).not.toEqual(initialQrScreenshot)
  })

  test('The library polls the status of this session. Upon "failed", the library informs the user that something went wrong and offers the option to try again, which leads to a new session.', async ({
    page,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 500,
      })
    })
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe(
      "Sorry, something went wrong",
    )
    expect(await mockRelyingPartyPage.getModalMessageText()).toBe(
      "This action was unsuccessful. This may have several reasons. Please try again.",
    )
    await expect(await mockRelyingPartyPage.getTryAgainButton()).toBeVisible()
    expect(await mockRelyingPartyPage.getWebsiteLink()).toBeDefined()
    await expect(await mockRelyingPartyPage.getCloseButton()).toBeVisible()
  })

  test('The library polls the status of this session. Upon "waiting for response", the library hides the QR code (when in cross-device) and tells the User to follow the instructions on their mobile device.', async ({
    page,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "WAITING_FOR_RESPONSE" }),
      })
    })
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe(
      "Follow the steps in your NL Wallet app",
    )
    await expect(await mockRelyingPartyPage.getHelpLink()).toBeVisible()
    expect(await mockRelyingPartyPage.getWebsiteLink()).toBeDefined()
    await expect(await mockRelyingPartyPage.getCancelButton()).toBeVisible()
  })

  test('The library polls the status of this session. Upon "expired", the library informs the user that the session was expired and offers them the option to try again, which leads to a new session.', async ({
    page,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "EXPIRED" }),
      })
    })
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe("Sorry, time is over")
    expect(await mockRelyingPartyPage.getModalMessageText()).toBe(
      "This action has been stopped because too much time has passed. This happens to keep your data safe. Please try again.",
    )
    await expect(await mockRelyingPartyPage.getHelpLink()).toBeVisible()
    await expect(await mockRelyingPartyPage.getTryAgainButton()).toBeVisible()
    await expect(await mockRelyingPartyPage.getCloseButton()).toBeVisible()
  })

  test('The library polls the status of this session. Upon "cancelled", the library confirms to the user that they have aborted the session and offers them the option to try again, which leads to a new session.', async ({
    page,
  }) => {
    await page.route("**/disclosure/sessions/**", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ status: "CANCELLED" }),
      })
    })
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe("Stopped")
    expect(await mockRelyingPartyPage.getModalMessageText()).toBe(
      "Because you have stopped, no data has been shared.",
    )
    await expect(await mockRelyingPartyPage.getHelpLink()).toBeVisible()
    await expect(await mockRelyingPartyPage.getTryAgainButton()).toBeVisible()
    await expect(await mockRelyingPartyPage.getCloseButton()).toBeVisible()
  })

  test("The library supports the following languages: Dutch, English. The language to be used is specified by the relying party.", async ({
    page,
  }, testInfo) => {
    await mockRelyingPartyPage.setDutchLanguage()
    await mockRelyingPartyPage.goToAmsterdamMunicipality()
    await mockRelyingPartyPage.openWalletLogin()
    const screenSizeName = testInfo.project.name.split("-")[1]
    if (screenSizeName === "Mobile") {
      expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe(
        "Op welk apparaat staat je NL Wallet app?",
      )
    } else {
      expect(await mockRelyingPartyPage.getModalMessageHeaderText()).toBe(
        "Scan de QR-code met je NL Wallet app",
      )
    }
  })
})
