package nativescreen.history

import util.NativeMobileActions

class HistoryDetailScreen : NativeMobileActions() {

    private val bottomBackButton = l10n.getString("generalBottomBackCta")
    private val reportProblemButton = l10n.getString("historyDetailScreenReportIssueCta")
    private val historyDetailScreenPurposeTitle = l10n.getString("historyDetailScreenPurposeTitle")
    private val historyDetailScreenTermsTitle = l10n.getString("historyDetailScreenTermsTitle")
    private val organizationButtonLabel = l10n.getString("organizationButtonLabel")

    fun visible() = elementWithTextVisible(bottomBackButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

    fun issuanceOrganizationVisible(organization: String) = elementContainingTextVisible(organization)

    fun disclosureOrganizationVisible(organization: String) = elementContainingTextVisible(organization)

    fun titleCorrectForIssuance(card: String) =
        elementWithTextVisible(l10n.getString("historyDetailScreenTitleForIssuance").replace("{card}", card))


    fun titleCorrectForLogin(organization: String) =
        elementWithTextVisible(l10n.getString("historyDetailScreenTitleForLogin").replace("{organization}", organization))

    fun openOrganizationScreen() = clickElementContainingText(organizationButtonLabel)

    fun attributeLabelVisible(label: String) = elementContainingTextVisible(label)

    fun reportProblemButtonVisible(): Boolean {
        scrollToElementContainingText(reportProblemButton)
        return elementContainingTextVisible(reportProblemButton)
    }

    fun reasonForSharingHeaderVisible() = elementWithTextVisible(historyDetailScreenPurposeTitle)

    fun reasonForSharingVisible(reason: String) = elementWithTextVisible(reason)

    fun termsVisible(): Boolean {
        scrollToElementWithText(historyDetailScreenTermsTitle)
        return elementWithTextVisible(historyDetailScreenTermsTitle)
    }
}
