package screen.issuance

import io.appium.java_client.AppiumBy
import org.openqa.selenium.WebElement
import util.MobileActions

class CardIssuanceScreen : MobileActions() {

    private val portraitImageLocator = AppiumBy.accessibilityId(l10n.getString("cardValueImage"))
    private val addCardButton = l10n.getPluralString("issuanceReviewCardsPageAcceptCta", 1, mapOf("cards" to "1"))
    private val add2CardsButton = l10n.getPluralString("issuanceReviewCardsPageAcceptCta", 2, mapOf("cards" to "2"))
    private val stopButton = l10n.getString("generalBottomBackCta")
    private val viewDetailsButton = l10n.getString("issuanceReviewCardsPageShowDetailsCta")
    private val dashboardButton = l10n.getString("issuanceSuccessPageCloseCta")
    private val renewCardsSectionTitle = l10n.getPluralString("issuanceReviewCardsPageRenewSectionTitle", 2, mapOf("cards" to "2"))

    fun portraitImageElement(): WebElement = findWebElement(portraitImageLocator)

    fun clickAddCardButton() {
        scrollToElementWithText(addCardButton)
        clickElementWithText(addCardButton)
    }

    fun addCardButtonVisible(timeoutInSeconds: Long = 20) = elementWithTextVisible(addCardButton, timeoutInSeconds)

    fun clickAdd2CardsButton() {
        scrollToElementWithText(add2CardsButton)
        clickElementWithText(add2CardsButton)
    }

    fun clickBackButton() {
        scrollToElementWithText(stopButton)
        clickElementWithText(stopButton)
    }

    fun viewDetailsOfCard(cardIdentifier: String) {
        scrollToEndOfScreen()
        findElementByPartialTextAndPartialSiblingText(viewDetailsButton, cardIdentifier).click()
    }

    fun labelVisible(label: String) = elementContainingTextVisible(label)

    fun dataVisible(data: String) = elementContainingTextVisible(data)

    fun organizationInSubtitleVisible(organization: String): Boolean {
        return elementWithTextVisible(l10n.getString("checkAttributesScreenSubtitle").replace("{issuer}",
            "$organization."
        ))
    }

    fun renewCardsSectionTitleVisible() = elementWithTextVisible(renewCardsSectionTitle)

    fun clickToDashboardButton() = clickElementWithText(dashboardButton)
}
