package screen.personalize

import util.MobileActions

class PersonalizeInformScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeInformPage")

    private val digidLoginButton = find.byValueKey("digidLoginCta")
    private val digidWebsiteButton = find.byValueKey("digidWebsiteCta")

    fun visible() = isElementVisible(screen)

    fun digidLoginButtonVisible() = isElementVisible(digidLoginButton)

    fun clickDigidLoginButton(switchToWebViewContext: Boolean = true) {
        clickElement(digidLoginButton)
        if (switchToWebViewContext) switchToWebViewContext()
    }

    fun clickDigidWebsiteButton() = clickElement(digidWebsiteButton)

    fun switchToWebView() = switchToWebViewContext()
}
