package screen.personalize

import util.MobileActions

class PersonalizeLoadingScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeLoadingPage")

    fun visible() = isElementVisible(screen, false)

    fun switchToApp() = switchToAppContext()
}
