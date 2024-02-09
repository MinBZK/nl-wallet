package screen.security

import util.MobileActions

class SecuritySetupCompletedScreen : MobileActions() {

    private val screen = find.byValueKey("setupSecurityCompletedPage")

    private val nextButton = find.byValueKey("primaryButtonCta")

    fun visible() = isElementVisible(screen)

    fun absent() = isElementAbsent(screen)

    fun clickNextButton() = clickElement(nextButton)
}
