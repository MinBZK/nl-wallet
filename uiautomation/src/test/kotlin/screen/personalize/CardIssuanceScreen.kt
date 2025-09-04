package screen.personalize

import helper.LocalizationHelper
import util.MobileActions

class CardIssuanceScreen : MobileActions() {

    private val addButton = find.byText(l10n.translate(LocalizationHelper.Translation.ADD_CARD))
    private val stopButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val viewDetailsButton = find.byText(l10n.getString("issuanceReviewCardsPageShowDetailsCta"))
    private val dashboardButton = find.byText(l10n.getString("issuanceSuccessPageCloseCta"))
    private val renewCardSectionTitle = find.byText(l10n.translate(LocalizationHelper.Translation.RENEW_CARD))
    private val scrollableElement = find.byType(ScrollableType.CustomScrollView.toString())

    fun clickAddButton() {
        scrollToEnd(scrollableElement)
        clickElement(addButton)
    }

    fun clickBackButton() {
        scrollToEnd(scrollableElement)
        clickElement(stopButton)
    }

    fun viewDetails() {
        clickElement(viewDetailsButton)
    }

    fun labelVisible(label: String): Boolean {
        return isElementVisible(find.byText(label))
    }

    fun dataVisible(data: String): Boolean {
        return isElementVisible(find.byText(data))
    }

    fun organizationInSubtitleVisible(organization: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("checkAttributesScreenSubtitle").replace("{issuer}",
            "$organization."
        )))
    }

    fun renewCardSectionTitleVisible() = isElementVisible(renewCardSectionTitle)

    fun clickToDashboardButton() = clickElement(dashboardButton)
}
