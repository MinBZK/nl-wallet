package screen.security

import util.MobileActions

class ChangePinSuccessScreen : MobileActions() {

    private val title = l10n.getString("changePinScreenSuccessTitle")
    private val description = l10n.getString("changePinScreenSuccessDescription")
    private val toSettingsButton = l10n.getString("changePinScreenToSettingsCta")

    fun visible() = elementWithTextVisible(title) && elementWithTextVisible(description)

    fun toSettings() = clickElementWithText(toSettingsButton)
}
