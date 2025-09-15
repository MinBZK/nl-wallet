package nativescreen.issuance

import helper.LocalizationHelper
import util.NativeMobileActions

class CardIssuanceScreen : NativeMobileActions() {

    private val addButton = l10n.translate(LocalizationHelper.Translation.ADD_CARD)
    private val stopButton = l10n.getString("generalBottomBackCta")
    private val viewDetailsButton = l10n.getString("issuanceReviewCardsPageShowDetailsCta")
    private val dashboardButton = l10n.getString("issuanceSuccessPageCloseCta")
    private val renewCardSectionTitle = l10n.translate(LocalizationHelper.Translation.RENEW_CARD)

    fun clickAddButton() {
        scrollToElementWithText(addButton)
        clickElementWithText(addButton)
    }

    fun clickBackButton() {
        scrollToElementWithText(stopButton)
        clickElementWithText(stopButton)
    }

    fun viewDetails() = clickElementWithText(viewDetailsButton)

    fun labelVisible(label: String): Boolean = elementContainingTextVisible(label)

    fun dataVisible(data: String): Boolean = elementContainingTextVisible(data)

    fun organizationInSubtitleVisible(organization: String): Boolean {
        return elementWithTextVisible(l10n.getString("checkAttributesScreenSubtitle").replace("{issuer}",
            "$organization."
        ))
    }

    fun renewCardSectionTitleVisible() = elementWithTextVisible(renewCardSectionTitle)

    fun clickToDashboardButton() = clickElementWithText(dashboardButton)
}
