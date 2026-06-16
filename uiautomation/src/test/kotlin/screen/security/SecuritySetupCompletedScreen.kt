package screen.security

import util.MobileActions

class SecuritySetupCompletedScreen : MobileActions() {

    private val setupSecurityCompletedPageTitle = l10n.getString("setupSecurityCompletedPageTitle")
    private val nextButton = l10n.getString("setupSecurityCompletedPageCreateWalletCta")
    private val closeButton = l10n.getString("generalClose")

    fun visible() = elementWithTextVisible(setupSecurityCompletedPageTitle)

    fun clickNextButton() {
        visible()
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        clickElementWithText(nextButton)
    }

    fun clickCloseButton() {
        visible()
        Thread.sleep(SET_FRAME_SYNC_MAX_WAIT_MILLIS)
        clickElementWithText(closeButton)
    }
}
