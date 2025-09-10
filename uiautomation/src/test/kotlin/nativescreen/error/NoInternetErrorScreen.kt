package nativescreen.error

import util.NativeMobileActions

class NoInternetErrorScreen : NativeMobileActions() {

    private val headline = l10n.getString("errorScreenNoInternetHeadline")
    private val description = l10n.getString("errorScreenNoInternetDescription")
    private val seeDetailsButton = l10n.getString("generalShowDetailsCta")
    private val tryAgainButton = l10n.getString("generalRetry")

    fun headlineVisible() = elementWithTextVisible(headline)

    fun descriptionVisible() = elementWithTextVisible(description)

    fun tryAgainButtonVisible() = elementWithTextVisible(tryAgainButton)

    fun appVersionLabelVisible(): Boolean {
        return elementContainingTextVisible(l10n.getString("generalVersionText"))
    }

    fun osVersionLabelVisible(): Boolean  {
        return elementContainingTextVisible(l10n.getString("generalOsVersionText"))
    }

    fun appConfigLabelVisible(): Boolean  {
        return elementContainingTextVisible(l10n.getString("generalConfigVersionText"))
    }

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
