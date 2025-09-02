package screen.personalize

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val screen = find.byValueKey("personalizePidPreviewPage")
    private val acceptButton = find.byValueKey("acceptButton")
    private val rejectButton = find.byValueKey("rejectButton")
    private val scrollableElement = find.byType(ScrollableType.CustomScrollView.toString())

    fun visible() = isElementVisible(screen)

    fun humanReadableCardDataVisible(cardData: String): Boolean {
        return isElementVisible(find.byText(cardData))
    }

    fun confirmButtonsVisible(): Boolean {
        scrollToEnd(scrollableElement)
        return isElementVisible(acceptButton) && isElementVisible(rejectButton)
    }

    fun clickAcceptButton() {
        scrollToEnd(scrollableElement)
        clickElement(acceptButton)
    }

    fun clickRejectButton() {
        scrollToEnd(scrollableElement)
        clickElement(rejectButton)
    }

    fun scrollToEnd() {
        scrollToEnd(scrollableElement)
    }
}
