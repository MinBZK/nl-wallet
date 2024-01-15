package screen.change_language

import util.MobileActions

class ChangeLanguageScreen : MobileActions() {

    private val screen = find.byValueKey("changeLanguageScreen")

    private val englishScreenTitle = find.byText("Select a language")
    private val dutchScreenTitle = find.byText("Kies een taal")

    private val englishButton = find.byText("English")
    private val dutchButton = find.byText("Nederlands")

    fun visible() = isElementVisible(screen)

    fun englishScreenTitleVisible() = isElementVisible(englishScreenTitle)

    fun dutchScreenTitleVisible() = isElementVisible(dutchScreenTitle)

    fun languageButtonsVisible(): Boolean = isElementVisible(englishButton) && isElementVisible(englishButton)

    fun clickEnglishButton() = clickElement(englishButton)

    fun clickDutchButton() = clickElement(dutchButton)
}
