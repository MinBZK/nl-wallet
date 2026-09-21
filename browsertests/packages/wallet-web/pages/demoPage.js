import { expect } from "@playwright/test"

export class DemoPage {
  constructor(page) {
    this.page = page

    // Top-level action buttons (Restored to ID locators since name-based matching failed)
    this.amsterdamMunicipalityButton = page.locator("#mijn_amsterdam_mdoc")
    this.xyzBankButton = page.locator("#xyz_bank_sd_jwt")
    this.marketplaceButton = page.locator("#online_marketplace")
    this.monkeyBikeButton = page.locator("#monkey_bike")
    this.universityButton = page.locator("#university_mdoc")

    // Modals and components
    this.walletModal = page.getByTestId("wallet_modal")
    this.nlWalletButtonTag = "nl-wallet-button"
    this.walletButtonContainer = page.locator(this.nlWalletButtonTag)

    // Components within the wallet button container
    this.nlWalletButton = this.walletButtonContainer.getByTestId("wallet_button")
    this.modalMessageHeader = this.walletButtonContainer.getByRole("heading", { level: 2 })
    this.modalMessageP = this.walletButtonContainer.locator(".modal p")

    this.helpLink = this.walletButtonContainer.getByTestId("help")
    this.retryButton = this.walletButtonContainer.getByTestId("retry_button")
    this.closeButton = this.walletButtonContainer.getByTestId("close_button")
    this.cancelButton = this.walletButtonContainer.getByTestId("cancel_button")
    this.websiteLink = this.walletButtonContainer.getByTestId("website_link")

    this.sameDeviceButton = this.walletButtonContainer.getByTestId("same_device_button")
    this.crossDeviceButton = this.walletButtonContainer.getByTestId("cross_device_button")
    this.qrCode = this.walletButtonContainer.getByTestId("qr")

    // Configuration/Controls
    this.languageSelector = page.locator('label[for="lang_toggle"]')
    this.dutchLanguageOption = page.locator('button[value="nl"]')
  }

  async goToAmsterdamMunicipality() {
    await this.amsterdamMunicipalityButton.click()
  }

  async goToXyzBank() {
    await this.xyzBankButton.click()
  }

  async goToMarketplace() {
    await this.marketplaceButton.click()
  }

  async goToMonkeyBike() {
    await this.monkeyBikeButton.click()
  }

  async goToUniversity() {
    await this.universityButton.click()
  }

  async getWalletButtonText() {
    return this.nlWalletButton.textContent()
  }

  async openWalletLogin() {
    await this.nlWalletButton.click()
  }

  async getWalletModal() {
    return this.walletModal
  }

  async getModalMessageHeaderText() {
    await this.waitForModalLoad()
    return this.modalMessageHeader.textContent()
  }

  async getModalMessageText() {
    await this.waitForModalLoad()
    return this.modalMessageP.textContent()
  }

  async getHelpLink() {
    await this.waitForModalLoad()
    return this.helpLink
  }

  async getTryAgainButton() {
    await this.waitForModalLoad()
    return this.retryButton
  }

  async getCloseButton() {
    await this.waitForModalLoad()
    return this.closeButton
  }

  async getCancelButton() {
    await this.waitForModalLoad()
    return this.cancelButton
  }

  async getSameDeviceButton() {
    await this.waitForModalLoad()
    return this.sameDeviceButton
  }

  async getCrossDeviceButton() {
    await this.waitForModalLoad()
    return this.crossDeviceButton
  }

  async getQrCode() {
    await this.waitForModalLoad()
    return this.qrCode
  }

  async getWebsiteLink() {
    await this.waitForModalLoad()
    return this.websiteLink
  }

  async getQrScreenshot() {
    await this.waitForModalLoad()
    return this.qrCode.screenshot()
  }

  async startCrossDeviceFlow() {
    await this.waitForModalLoad()
    if (await this.crossDeviceButton.isVisible()) {
      await this.crossDeviceButton.click()
    }
  }

  async setDutchLanguage() {
    await this.languageSelector.click()
    await this.dutchLanguageOption.click()
  }

  async waitForModalLoad() {
    await expect(this.modalMessageHeader).not.toContainText(/(Please wait|Even geduld)/)
  }
}
