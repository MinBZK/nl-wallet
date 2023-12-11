package screen.personalize

import util.MobileActions

class PersonalizeAuthenticatingWithDigidScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeAuthenticatingWithDigidPage")

    private val text = find.byText("Login bij DigiD")

    fun visible() = isElementVisible(text, false)

    fun switchToApp() = switchToAppContext()
}
