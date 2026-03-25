export class PortalPage {
  constructor(page) {
    this.page = page

    this.title = "h1"
    this.revocationSubmitButton = ".btn-delete"
    this.revocationCancelButton = ".btn-cancel"
    this.helpLink = ".help-link"
    this.universityButton = "#university_mdoc"
    this.successMessage = "#success_message"
    this.revocationCodeInput = "#deletion-code"

    this.languageSelector = 'label[for="lang_toggle"]'
    this.dutchLanguageOption = 'button[value="nl"]'
    this.englishLanguageOption = 'button[value="en"]'
  }

  async enterRevocationCode(code) {
    await this.page.locator(this.revocationCodeInput).fill(code)
  }

  async submitRevocation() {
    await Promise.all([this.page.waitForLoadState("load"), this.page.locator(this.revocationSubmitButton).click()])
  }

  async cancelRevocation() {
    await this.page.locator(this.revocationCancelButton).click()
  }

  async getHelpLink() {
    return this.page.locator(this.helpLink)
  }

  async getTitle() {
    return this.page.locator(this.title).innerText()
  }

  async setDutchLanguage() {
    await this.page.locator(this.languageSelector).click()
    await this.page.locator(this.dutchLanguageOption).click()
  }

  async setEnglishLanguage() {
    await this.page.locator(this.languageSelector).click()
    await this.page.locator(this.englishLanguageOption).click()
  }
}
