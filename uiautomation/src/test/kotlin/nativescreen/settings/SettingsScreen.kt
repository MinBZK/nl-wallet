package nativescreen.settings

import util.NativeMobileActions

class SettingsScreen : NativeMobileActions() {

    private val screenTitle = l10n.getString("settingsScreenTitle")
    private val changePinButton = l10n.getString("settingsScreenChangePinCta")
    private val changeLanguageButton = l10n.getString("settingsScreenChangeLanguageCta")
    private val clearDataButton = l10n.getString("settingsScreenClearDataCta")

    fun visible() = elementWithTextVisible(screenTitle)

    fun settingsButtonsVisible() =
        elementWithTextVisible(changePinButton) && elementWithTextVisible(changeLanguageButton) && elementWithTextVisible(clearDataButton)

    fun clickChangeLanguageButton() = clickElementWithText(changeLanguageButton)

    fun clickClearDataButton() = clickElementWithText(clearDataButton)
}
