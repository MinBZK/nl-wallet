package screen.history

import util.MobileActions

class HistoryDetailScreen : MobileActions() {

    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    private val reportProblemButton = l10n.getString("historyDetailScreenReportIssueCta")
    private val historyDetailScreenPurposeTitle = l10n.getString("historyDetailScreenPurposeTitle")
    private val historyDetailScreenAboutOrganizationCta = l10n.getString("historyDetailScreenAboutOrganizationCta").replace("{organization}",  "")

    fun visible() = elementWithTextVisible(bottomBackButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

    fun issuanceOrganizationVisible(organization: String) = elementContainingTextVisible(organization)

    fun disclosureOrganizationVisible(organization: String) = elementContainingTextVisible(organization)

    fun titleCorrectForIssuance(card: String) =
        elementWithTextVisible(l10n.getString("historyDetailScreenTitleForCardIssued").replace("{card}", card))


    fun titleCorrectForLogin(organization: String) =
        elementWithTextVisible(l10n.getString("historyDetailScreenTitleForLogin").replace("{organization}", organization))

    fun openOrganizationScreen() = clickElementContainingText(historyDetailScreenAboutOrganizationCta)

    fun attributeLabelVisible(label: String) = elementContainingTextVisible(label)

    fun reportProblemButtonVisible(): Boolean {
        scrollToElementContainingText(reportProblemButton)
        return elementContainingTextVisible(reportProblemButton)
    }

    fun reasonForSharingHeaderVisible() = elementWithTextVisible(historyDetailScreenPurposeTitle)

    fun reasonForSharingVisible(reason: String): Boolean {
        // TODO PVW-6101 check for purpose
        return true
    }
}
