package screen.privacy

import util.MobileActions

class PrivacyPolicyScreen : MobileActions() {

    private val screen = find.byValueKey("privacyPolicyScreen")

    fun visible() = isElementVisible(screen)
}
