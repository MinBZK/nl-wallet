package screen.settings

import util.MobileActions

class SettingsScreen : MobileActions() {

    private val screen = find.byValueKey("settingsScreen")

    private val changePinButton = find.byText(l10n.getString("settingsScreenChangePinCta"))
    private val setupBiometricsButton = find.byText("Open met gezichts-scan of vingerafdruk")
    private val changeLanguageButton = find.byText(l10n.getString("settingsScreenChangeLanguageCta"))
    private val clearDataButton = find.byText(l10n.getString("settingsScreenClearDataCta"))

    fun visible() = isElementVisible(screen)

    fun settingsButtonsVisible(): Boolean =
        isElementVisible(changePinButton) && isElementVisible(changeLanguageButton) && isElementVisible(clearDataButton)

    fun clickChangeLanguageButton() = clickElement(changeLanguageButton)

    fun clickClearDataButton() = clickElement(clearDataButton)
}
