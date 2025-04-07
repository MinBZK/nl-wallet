import { expect } from "@playwright/test"

export class MockRelyingPartyPage {
  constructor(page) {
    this.page = page

    this.amsterdamMunicipalityButton = "#mijn_amsterdam"
    this.xyzBankButton = "#xyz_bank"
    this.marketplaceButton = "#online_marketplace"
    this.monkeyBikeButton = "#monkey_bike"

    this.nlWalletButtonTag = "nl-wallet-button"
    this.nlWalletButton = ".nl-wallet-button"
    this.modalMessageHeader = ".modal h2"
    this.modalMessageP = ".modal p"

    this.helpLink = 'a[data-testid="help"]'
    this.retryButton = 'button[data-testid="retry_button"]'
    this.closeButton = 'button[data-testid="close_button"]'
    this.cancelButton = 'button[data-testid="cancel_button"]'
    this.websiteLink = 'section[data-testid="website_link"'

    this.sameDeviceButton = 'a[data-testid="same_device_button"]'
    this.crossDeviceButton = 'button[data-testid="cross_device_button"]'
    this.qrCode = 'div[data-testid="qr"]'

    this.languageSelector = 'label[for="lang_toggle"]'
    this.dutchLanguageOption = 'button[value="nl"]'
  }

  async goToAmsterdamMunicipality() {
    await this.page.locator(this.amsterdamMunicipalityButton).click()
  }

  async goToXyzBank() {
    await this.page.locator(this.xyzBankButton).click()
  }

  async goToMarketplace() {
    await this.page.locator(this.marketplaceButton).click()
  }

  async goToMonkeyBike() {
    await this.page.locator(this.monkeyBikeButton).click()
  }

  async getWalletButtonText() {
    return this.page.locator(this.nlWalletButtonTag).locator(this.nlWalletButton).textContent()
  }

  async openWalletLogin() {
    await this.page.locator(this.nlWalletButtonTag).locator(this.nlWalletButton).click()
  }

  async getModalMessageHeaderText() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.modalMessageHeader).textContent()
  }

  async getModalMessageText() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.modalMessageP).textContent()
  }

  async getHelpLink() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.helpLink)
  }

  async getTryAgainButton() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.retryButton)
  }

  async getCloseButton() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.closeButton)
  }

  async getCancelButton() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.cancelButton)
  }

  async getSameDeviceButton() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.sameDeviceButton)
  }

  async getCrossDeviceButton() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.crossDeviceButton)
  }

  async getQrCode() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.qrCode)
  }

  async getWebsiteLink() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.websiteLink)
  }

  async getQrScreenshot() {
    await this.waitForModalLoad()
    return this.page.locator(this.nlWalletButtonTag).locator(this.qrCode).screenshot()
  }

  async startCrossDeviceFlow() {
    await this.waitForModalLoad()
    const button = this.page.locator(this.nlWalletButtonTag).locator(this.crossDeviceButton)
    if (await button.isVisible()) {
      await button.click()
    }
  }

  async setDutchLanguage() {
    await this.page.locator(this.languageSelector).click()
    await this.page.locator(this.dutchLanguageOption).click()
  }

  async waitForModalLoad() {
    await expect(this.page.locator(this.nlWalletButtonTag).locator(this.modalMessageHeader)).not.toContainText(
      /(Please wait|Even geduld)/,
    )
  }
}
