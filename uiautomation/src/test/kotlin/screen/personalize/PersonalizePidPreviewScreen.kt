package screen.personalize

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val screen = find.byValueKey("personalizePidPreviewPage")

    private val birthText = find.byText("24 maart 2000")
    private val streetNameText = find.byText("Van Wijngaerdenstraat")
    private val postcodeText = find.byText("2596TW")
    private val houseNumberText = find.byText("1")

    private val acceptButton = find.byValueKey("acceptButton")
    private val rejectButton = find.byValueKey("rejectButton")

    fun visible() = isElementVisible(screen)

    fun humanReadablePidDataVisible() =
        isElementVisible(birthText) && isElementVisible(streetNameText) && isElementVisible(postcodeText) && isElementVisible(houseNumberText)

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
