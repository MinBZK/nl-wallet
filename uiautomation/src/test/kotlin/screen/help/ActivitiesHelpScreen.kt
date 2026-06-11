package screen.help

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

    fun clickSomethingElseButton() = clickElementWithText(somethingElseButton)

    fun clickFirstHelpGroupButton() {
        when (platformName()) {
            "ANDROID" -> driver.findElement(
                By.xpath("//*[@content-desc='$helpHeader']//android.widget.Button[1]")
            ).click()
            "IOS" -> driver.findElement(
                By.xpath("//*[@name='$helpHeader']/following-sibling::XCUIElementTypeButton[1]")
            ).click()
        }
    }
}
