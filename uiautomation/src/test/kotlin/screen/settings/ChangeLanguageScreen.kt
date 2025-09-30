package screen.settings

import util.MobileActions

class ChangeLanguageScreen : MobileActions() {

    private val screenTitle = l10n.getString("changeLanguageScreenTitle")
    private val englishScreenTitle = "Select a language"
    private val dutchScreenTitle = "Kies een taal"
    private val englishButton = "English"
    private val dutchButton = "Nederlands"

    fun visible() = elementWithTextVisible(screenTitle)

    fun englishScreenTitleVisible() = elementWithTextVisible(englishScreenTitle)

    fun dutchScreenTitleVisible() = elementWithTextVisible(dutchScreenTitle)

    fun languageButtonsVisible() = elementWithTextVisible(englishButton) && elementWithTextVisible(englishButton)

    fun clickEnglishButton() = clickElementWithText(englishButton)

    fun clickDutchButton() = clickElementWithText(dutchButton)
}
