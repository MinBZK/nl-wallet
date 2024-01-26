package screen.personalize

import util.MobileActions

class PersonalizeInformScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeInformPage")

    private val loginWithDigidButton = find.byValueKey("loginWithDigidCta")
    private val noDigidButton = find.byValueKey("noDigidCta")

    fun visible() = isElementVisible(screen)

    fun loginWithDigidButtonVisible() = isElementVisible(loginWithDigidButton)

    fun clickLoginWithDigidButton(switchToWebViewContext: Boolean = true) {
        clickElement(loginWithDigidButton)
        if (switchToWebViewContext) switchToWebViewContext()
    }

    fun clickNoDigidButton() = clickElement(noDigidButton)
}
