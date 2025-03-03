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

  getDeeplink() {
    return this.page.locator(this.deeplink)
  }

  getStoreBanners() {
    return this.page.locator(this.storeBanners)
  }

  getHelpLink() {
    return this.page.locator(this.helpLink)
  }
}
