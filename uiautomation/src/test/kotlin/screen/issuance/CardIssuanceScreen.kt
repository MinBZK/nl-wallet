package screen.issuance

import helper.LocalizationHelper
import util.MobileActions

class CardIssuanceScreen : MobileActions() {

    private val addCardButton = l10n.getPluralString("issuanceReviewCardsPageAcceptCta", 1, mapOf("cards" to "1"))
    private val add2CardsButton = l10n.getPluralString("issuanceReviewCardsPageAcceptCta", 2, mapOf("cards" to "2"))
    private val stopButton = l10n.getString("generalBottomBackCta")
    private val viewDetailsButton = l10n.getString("issuanceReviewCardsPageShowDetailsCta")
    private val dashboardButton = l10n.getString("issuanceSuccessPageCloseCta")
    private val renewCardSectionTitle = l10n.getPluralString("issuanceReviewCardsPageRenewSectionTitle", 1, mapOf("cards" to "1"))

    fun clickAddCardButton() {
        scrollToElementWithText(addCardButton)
        clickElementWithText(addCardButton)
    }

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

    fun renewCardSectionTitleVisible() = elementWithTextVisible(renewCardSectionTitle)

    fun clickToDashboardButton() = clickElementWithText(dashboardButton)
}
