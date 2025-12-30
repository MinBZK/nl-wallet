package screen.error

import util.MobileActions

class InvalidIssuanceULErrorScreen : MobileActions() {

    private val headline = l10n.getString("issuanceRelyingPartyErrorTitle")
    private val closeButton = l10n.getString("generalClose")

    val errorDetails = ErrorDetailsBottomSheet()

    fun headlineVisible() = elementWithTextVisible(headline)

    fun closeButtonVisible() = elementWithTextVisible(closeButton)
}
