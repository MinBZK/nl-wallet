package screen.personalize

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val screen = find.byValueKey("personalizePidPreviewPage")

    private val birthText = find.byText("2000-03-24")
    private val streetNameText = find.byText("Groenewoudsedijk")
    private val postcodeText = find.byText("3528BG")
    private val houseNumberText = find.byText("51")

    private val acceptButton = find.byValueKey("acceptButton")
    private val rejectButton = find.byValueKey("rejectButton")

    fun visible() = isElementVisible(screen)

    fun humanReadablePidDataVisible() =
        isElementVisible(birthText) && isElementVisible(streetNameText) && isElementVisible(postcodeText) && isElementVisible(
            houseNumberText
        )

    fun confirmButtonsVisible(): Boolean {
        scrollToEnd(ScrollableType.CustomScrollView)
        return isElementVisible(acceptButton) && isElementVisible(rejectButton)
    }

    fun clickAcceptButton() {
        scrollToEnd(ScrollableType.CustomScrollView)
        clickElement(acceptButton)
    }

    fun clickRejectButton() {
        scrollToEnd(ScrollableType.CustomScrollView)
        clickElement(rejectButton)
    }
}
