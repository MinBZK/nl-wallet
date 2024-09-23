package screen.personalize

import util.MobileActions

class PersonalizeAuthenticatingWithDigidScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeAuthenticatingWithDigidPage")

    fun visible() = isElementVisible(screen, false)
}
