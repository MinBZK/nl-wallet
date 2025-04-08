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
        return Regex("""\b\d+\.\d+\.\d+\b""").containsMatchIn(getTextFromElementContainingText(l10n.getString("generalVersionText")))
    }

    fun osVersionVisible(): Boolean  {
        return Regex(""".*\d+.*""").containsMatchIn(getTextFromElementContainingText(l10n.getString("generalOsVersionText")))
    }

    fun appConfigVisible(): Boolean  {
        return Regex(""".*\d+$""").containsMatchIn(getTextFromElementContainingText(l10n.getString("generalConfigVersionText")))
    }

    fun seeDetails() = clickElement(seeDetailsButton, false)
}
