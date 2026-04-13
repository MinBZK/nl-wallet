package screen.permissions

import util.MobileActions

class BluetoothPermissionScreen : MobileActions() {

    private val title = l10n.getString("qrShowBluetoothPermissionTitle")
    private val description = l10n.getString("qrShowBluetoothPermissionDescription")
    private val openSettingsCta = l10n.getString("qrShowBluetoothPermissionSettingsCta")

    fun visible() = elementWithTextVisible(title)

    fun descriptionVisible() = elementWithTextVisible(description)

    fun openSettingsButtonVisible() = elementWithTextVisible(openSettingsCta)
}
