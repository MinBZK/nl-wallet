package screen.error

import util.MobileActions

class NoInternetErrorScreen : MobileActions() {

    private val headline = find.byText(l10n.getString("errorScreenNoInternetHeadline"))
    private val description = find.byText(l10n.getString("errorScreenNoInternetDescription"))
    private val seeDetailsButton = find.byText(l10n.getString("generalShowDetailsCta"))
    private val tryAgainButton = find.byText(l10n.getString("generalRetry"))

    fun headlineVisible() = isElementVisible(headline, false)

    fun descriptionVisible() = isElementVisible(description, false)

    fun tryAgainButtonVisible() = isElementVisible(tryAgainButton, false)

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

    fun seeDetails() = clickElement(seeDetailsButton, false)
}
