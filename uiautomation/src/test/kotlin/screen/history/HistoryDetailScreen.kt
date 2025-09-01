package screen.history

import util.MobileActions

class HistoryDetailScreen : MobileActions() {

    private val screen = find.byValueKey("historyDetailScreen")

    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val reportProblemButton = find.byText(l10n.getString("historyDetailScreenReportIssueCta"))
    private val historyDetailScreenPurposeTitle = find.byText(l10n.getString("historyDetailScreenPurposeTitle"))
    private val historyDetailScreenTermsTitle = find.byText(l10n.getString("historyDetailScreenTermsTitle"))
    private val organizationButtonLabel = find.byText(l10n.getString("organizationButtonLabel"))
    private val scrollableElement = find.byType(ScrollableType.CustomScrollView.toString())

    fun visible() = isElementVisible(screen, false)

    fun clickBottomBackButton() = clickElement(bottomBackButton, false)

    fun issuanceOrganizationVisible(organization: String): Boolean {
        return isElementVisible(find.byText(organization), false)
    }

    fun disclosureOrganizationVisible(organization: String): Boolean {
        scrollToEnd(scrollableElement)
        val link = l10n.getString("historyDetailScreenAboutOrganizationCta").replace("{organization}", organization)
        return isElementVisible(find.byText(link), false)
    }

    fun titleCorrectForIssuance(card: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("historyDetailScreenTitleForIssuance").replace("{card}", card)), false)
    }


    fun titleCorrectForLogin(organization: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("historyDetailScreenTitleForLogin").replace("{organization}", organization)), false)
    }

    fun openOrganizationScreen() = clickElement(organizationButtonLabel)

    fun attributeLabelVisible(label: String): Boolean {
        return isElementVisible(find.byText(label))
    }

    fun reportProblemButtonVisible() = isElementVisible(reportProblemButton)

    fun reasonForSharingHeaderVisible() = isElementVisible(historyDetailScreenPurposeTitle)

    fun reasonForSharingVisible(reason: String): Boolean {
        return isElementVisible(find.byText(reason))
    }

    fun termsVisible(): Boolean {
        scrollToEnd(scrollableElement)
        return isElementVisible(historyDetailScreenTermsTitle)
    }
}
