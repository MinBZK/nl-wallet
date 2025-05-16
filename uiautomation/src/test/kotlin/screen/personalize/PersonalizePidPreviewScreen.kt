package screen.personalize

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val screen = find.byValueKey("personalizePidPreviewPage")
    private val acceptButton = find.byValueKey("acceptButton")
    private val rejectButton = find.byValueKey("rejectButton")

    fun visible() = isElementVisible(screen)

    fun humanReadableCardDataVisible(cardData: String): Boolean {
        return isElementVisible(find.byText(cardData))
    }

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
