package screen.help

import domain.Platform
import org.openqa.selenium.By
import util.MobileActions

class ActivitiesHelpScreen : MobileActions() {

    private val title = l10n.getString("menuScreenTourCta")
    private val somethingElseButton = l10n.getString("helpTopicScreenSomethingElseCta")
    private val cardActivitiesButton = l10n.getString("cardHistoryScreenTitle")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementContainingTextVisible(title)

    fun clickCardActivitiesButton() = clickElementContainingText(cardActivitiesButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)


    fun clickSomethingElseButton() {
        scrollToElementWithText(somethingElseButton)
        clickElementWithText(somethingElseButton)
    }

    fun clickFirstHelpGroupButton() {
        when (platform()) {
            Platform.ANDROID -> driver.findElement(
                By.xpath("(//*[@scrollable='true']//*[@clickable='true'])[1]")
            ).click()
            Platform.IOS -> driver.findElement(
                By.xpath("//XCUIElementTypeScrollView//XCUIElementTypeButton[1]")
            ).click()
        }
    }
}
