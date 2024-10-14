package screen.disclosure

import util.MobileActions

class DisclosureApproveOrganizationScreen : MobileActions() {

    private val loginAmsterdamTitleText = find.byText("Wil je inloggen bij Gemeente Amsterdam?")

    fun loginAmsterdamTitleVisible() = isElementVisible(loginAmsterdamTitleText)
}
