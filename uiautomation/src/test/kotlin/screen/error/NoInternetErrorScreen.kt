package screen.error

import util.MobileActions

class NoInternetErrorScreen : MobileActions() {

    private val headline = l10n.getString("errorScreenNoInternetHeadline")
    private val description = l10n.getString("errorScreenNoInternetDescription")
    private val tryAgainButton = l10n.getString("generalRetry")
    private val closeButton = l10n.getString("generalClose")

    val errorDetails = ErrorDetailsBottomSheet()

    fun headlineVisible() = elementWithTextVisible(headline)

    fun descriptionVisible() = elementWithTextVisible(description)

    fun tryAgainButtonVisible() = elementWithTextVisible(tryAgainButton)

    fun clickCloseButton() = clickElementWithText(closeButton)
}
