package screen.personalize

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val screen = find.byValueKey("personalizePidPreviewPage")

    private val birthText = find.byText("24 maart 2000")
    private val addressText = find.byText("Groenewoudsedijk 51, 3528BG Utrecht")

    private val acceptButton = find.byValueKey("acceptButton")
    private val rejectButton = find.byValueKey("rejectButton")

    fun visible() = isElementVisible(screen)

    fun humanReadablePidDataVisible() =
        isElementVisible(birthText) && isElementVisible(addressText)

    fun confirmButtonsVisible() = isElementVisible(acceptButton) && isElementVisible(rejectButton)

    fun clickAcceptButton() = clickElement(acceptButton)

    fun clickRejectButton() = clickElement(rejectButton)

    fun switchToApp() = switchToAppContext()
}
