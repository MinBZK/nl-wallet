package screen.disclosure

import util.MobileActions

class BleDisconnectedScreen : MobileActions() {

    private val title = l10n.getString("disclosureDisconnectedPageTitle")

    fun visible() = elementWithTextVisible(title)
}
