package screen.error

import util.MobileActions

class InvalidIssuanceULErrorScreen : MobileActions() {

    private val headline = l10n.getString("issuanceRelyingPartyErrorTitle")
    private val seeDetailsButton = l10n.getString("generalShowDetailsCta")
    private val closeButton = l10n.getString("generalClose")

    fun headlineVisible() = elementWithTextVisible(headline)

    fun closeButtonVisible() = elementWithTextVisible(closeButton)

    fun appVersionLabelVisible() = elementContainingTextVisible(l10n.getString("generalVersionText"))

    fun osVersionLabelVisible() = elementContainingTextVisible(l10n.getString("generalOsVersionText"))

    fun appConfigLabelVisible()  = elementContainingTextVisible(l10n.getString("generalConfigVersionText"))

    fun appVersionVisible(): Boolean  {
        return getTextFromElementContainingText(l10n.getString("generalVersionText"))
            ?.contains("""\b\d+\.\d+\.\d+\b""".toRegex()) ?: false
    }

    fun osVersionVisible(): Boolean  {
        return getTextFromElementContainingText(l10n.getString("generalOsVersionText"))
            ?.contains(""".*\d+.*""".toRegex()) ?: false
    }

    fun appConfigVisible(): Boolean  {
        return getTextFromElementContainingText(l10n.getString("generalConfigVersionText"))
            ?.contains(""".*\d+$""".toRegex()) ?: false
    }

    fun seeDetails() = clickElementWithText(seeDetailsButton)
}
