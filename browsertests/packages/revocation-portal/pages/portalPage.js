export class PortalPage {
  constructor(page) {
    this.page = page

    // Core page components initialized as locators using exact existing CSS selectors
    this.title = page.getByRole("heading", { level: 1 })
    this.revocationSubmitButton = page.locator(".btn-delete")
    this.revocationCancelButton = page.locator(".btn-cancel")
    this.helpLink = page.locator(".help-link")
    this.universityButton = page.locator("#university_mdoc")
    this.successMessage = page.locator("#success_message")
    this.revocationCodeInput = page.locator("#deletion-code")

    // Configuration / Localization controls
    this.languageSelector = page.locator('label[for="lang_toggle"]')
    this.dutchLanguageOption = page.locator('button[value="nl"]')
    this.englishLanguageOption = page.locator('button[value="en"]')
  }

  async enterRevocationCode(code) {
    await this.revocationCodeInput.fill(code)
  }

  async submitRevocation() {
    await Promise.all([this.page.waitForLoadState("load"), this.revocationSubmitButton.click()])
  }

  async cancelRevocation() {
    await this.revocationCancelButton.click()
  }

  async getHelpLink() {
    return this.helpLink
  }

  async getTitle() {
    return this.title.innerText()
  }

  async setDutchLanguage() {
    await this.languageSelector.click()
    await this.dutchLanguageOption.click()
  }

  async setEnglishLanguage() {
    await this.languageSelector.click()
    await this.englishLanguageOption.click()
  }
}
