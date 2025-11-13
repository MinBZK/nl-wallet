package screen.security

import util.MobileActions

class RecoverPinSuccesScreen : MobileActions() {

    private val title = l10n.getString("recoverPinSuccessPageTitle")
    private val toOverviewButton = l10n.getString("recoverPinSuccessPageToOverviewCta")

    fun visible() = elementWithTextVisible(title)

    fun clickToOverviewButton() = clickElementWithText(toOverviewButton)
}
