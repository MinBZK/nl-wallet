package screen.personalize

import util.MobileActions

class PersonalizeInformScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeInformPage")

    private val digidLoginButton = find.byText(l10n.getString("walletPersonalizeIntroPageLoginWithDigidCta"))
    private val digidWebsiteButton = find.byText(l10n.getString("walletPersonalizeDigidErrorPageDigidWebsiteCta"))

    fun visible() = isElementVisible(screen)

    fun clickDigidLoginButton(switchToWebViewContext: Boolean = true) {
        clickElement(digidLoginButton)
        if (switchToWebViewContext) switchToWebViewContext()
    }

    fun clickDigidWebsiteButton() = clickElement(digidWebsiteButton)

    fun switchToWebView() = switchToWebViewContext()
}
