package screen.personalize

import util.MobileActions

class PersonalizeConfirmPinScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeConfirmPinPage")

    fun visible() = isElementVisible(screen)
}
