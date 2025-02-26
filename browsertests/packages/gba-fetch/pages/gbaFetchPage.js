export class GbaFetchPage {
  constructor(page) {
    this.page = page

    this.pageTitle = "h1"
    this.statusInfoLabel = 'label[for="preloaded_count"]'
    this.StatusInfoPreloadedCount = "#preloaded_count"

    this.bsnInput = "#bsn"
    this.repeatBsnInput = "#repeat_bsn"
    this.preloadButton = 'form[action="/"] button'

    this.clearDataLabel = 'label[for="confirmation_text"]'
    this.clearDataInput = "#confirmation_text"
    this.clearDataButton = 'form[action="/clear"] button'

    this.result = "#result"
    this.backButton = 'a[href="/"]'
  }

  async getPageTitle() {
    return this.page.textContent(this.pageTitle)
  }

  async getStatusInfoLabel() {
    return this.page.textContent(this.statusInfoLabel)
  }

  async getStatusInfoPreloadedCount() {
    return this.page.textContent(this.StatusInfoPreloadedCount)
  }

  async getBsnInput() {
    return this.page.locator(this.bsnInput)
  }

  async getRepeatBsnInput() {
    return this.page.locator(this.repeatBsnInput)
  }

  async getPreloadButton() {
    return this.page.locator(this.preloadButton)
  }

  async getClearDataLabel() {
    return this.page.locator(this.clearDataLabel)
  }

  async getClearDataLabelText() {
    return this.page.textContent(this.clearDataLabel)
  }

  async getClearDataInput() {
    return this.page.locator(this.clearDataInput)
  }

  async getClearDataButton() {
    return this.page.locator(this.clearDataButton)
  }

  async enterBsn(bsn) {
    await this.page.locator(this.bsnInput).fill(bsn)
  }

  async repeatBsn(bsn) {
    await this.page.locator(this.repeatBsnInput).fill(bsn)
  }

  async preload() {
    await this.page.locator(this.preloadButton).click()
  }

  async enterClearDataText(text) {
    await this.page.locator(this.clearDataInput).fill(text)
  }

  async clearData() {
    await this.page.locator(this.clearDataButton).click()
  }

  async getResult() {
    return this.page.textContent(this.result)
  }

  async goBack() {
    await this.page.locator(this.backButton).click()
  }

  async getBsnValidationMessage() {
    return await this.page.locator(this.bsnInput).evaluate((el) => el.validationMessage)
  }
}
