export class FallbackPage {
  constructor(page) {
    this.page = page

    this.deeplink = "#deeplink"
    this.pageTitle = "h1"
    this.storeBanners = ".store-banners"
    this.helpLink = "footer .button-link"
  }

  async getPageTitle() {
    return this.page.textContent(this.pageTitle)
  }

  async getDeeplink() {
    return this.page.locator(this.deeplink)
  }

  async getStoreBanners() {
    return this.page.locator(this.storeBanners)
  }

  async getHelplink() {
    return this.page.locator(this.storeBanners)
  }
}
