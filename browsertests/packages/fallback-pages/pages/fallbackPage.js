export class FallbackPage {
  constructor(page) {
    this.page = page

    // Fallback page components using Playwright locators
    this.deeplink = page.locator("#deeplink")
    this.pageTitle = page.getByRole("heading", { level: 1 })
    this.storeBanners = page.locator(".store-banners")
    this.helpLink = page.locator("footer .button-link")
  }

  async getPageTitle() {
    return this.pageTitle.textContent()
  }

  getDeeplink() {
    return this.deeplink
  }

  getStoreBanners() {
    return this.storeBanners
  }

  getHelpLink() {
    return this.helpLink
  }
}
