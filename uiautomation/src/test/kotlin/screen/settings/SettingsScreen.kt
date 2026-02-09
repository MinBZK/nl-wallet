package screen.settings

import util.MobileActions

class SettingsScreen : MobileActions() {

    private val screenTitle = l10n.getString("settingsScreenTitle")
    private val changePinButton = l10n.getString("settingsScreenChangePinCta")
    private val changeLanguageButton = l10n.getString("settingsScreenChangeLanguageCta")
    private val clearDataButton = l10n.getString("settingsScreenClearDataCta")
    private val backButton = l10n.getString("generalBottomBackCta")
    private val notificationsButton = l10n.getString("settingsScreenManageNotificationsCta")
    private val revocationCodeButton = l10n.getString("settingsScreenShowRevocationCodeCta")

    fun visible() = elementWithTextVisible(screenTitle)

    fun settingsButtonsVisible() =
        elementWithTextVisible(changePinButton) && elementWithTextVisible(changeLanguageButton) && elementWithTextVisible(clearDataButton)

    fun clickChangeLanguageButton() = clickElementWithText(changeLanguageButton)

    fun clickClearDataButton() = clickElementWithText(clearDataButton)

    fun clickChangePinButton() = clickElementWithText(changePinButton)

    fun clickBackButton() = clickElementWithText(backButton)

    fun clickNotificationsButton() = clickElementWithText(notificationsButton)

    fun clickRevocationCodeButton() = clickElementWithText(revocationCodeButton)
}
