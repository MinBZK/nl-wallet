package screen.help

import domain.Platform
import org.openqa.selenium.By
import util.MobileActions

class ActivitiesHelpScreen : MobileActions() {

    private val title = l10n.getString("menuScreenTourCta")
    private val somethingElseButton = l10n.getString("helpTopicScreenSomethingElseCta")
    private val cardActivitiesButton = l10n.getString("cardHistoryScreenTitle")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    private val helpHeader = l10n.getString("helpScreenGroupTitleHelp")
    private val infoHeader = l10n.getString("helpScreenGroupTitleInformation")

    fun visible() = elementContainingTextVisible(title)

    fun clickCardActivitiesButton() = clickElementContainingText(cardActivitiesButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

    fun helpAndInfoHeadersVisible() = elementWithTextVisible(helpHeader) && elementWithTextVisible(infoHeader)

    fun clickSomethingElseButton() {
        scrollToElementWithText(somethingElseButton)
        clickElementWithText(somethingElseButton)
    }

    fun clickFirstHelpGroupButton() {
        when (platform()) {
            Platform.ANDROID -> driver.findElement(
                By.xpath("//*[@content-desc='$helpHeader']//android.widget.Button[1]")
            ).click()
            Platform.IOS -> driver.findElement(
                By.xpath("//*[@name='$helpHeader']/following-sibling::XCUIElementTypeButton[1]")
            ).click()
        }
    }
}
