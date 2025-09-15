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

    fun issuanceOrganizationVisible(organization: String): Boolean = elementWithTextVisible(organization)

    fun disclosureOrganizationVisible(organization: String): Boolean {
        val link = l10n.getString("historyDetailScreenAboutOrganizationCta").replace("{organization}", organization)
        scrollToElementWithText(link)
        return elementWithTextVisible(link)
    }

    fun titleCorrectForIssuance(card: String): Boolean =
        elementWithTextVisible(l10n.getString("historyDetailScreenTitleForIssuance").replace("{card}", card))


    fun titleCorrectForLogin(organization: String): Boolean =
        elementWithTextVisible(l10n.getString("historyDetailScreenTitleForLogin").replace("{organization}", organization))

    fun openOrganizationScreen() = clickElementWithText(organizationButtonLabel)

    fun attributeLabelVisible(label: String): Boolean = elementWithTextVisible(label)

    fun reportProblemButtonVisible() = elementWithTextVisible(reportProblemButton)

    fun reasonForSharingHeaderVisible() = elementWithTextVisible(historyDetailScreenPurposeTitle)

    fun reasonForSharingVisible(reason: String): Boolean = elementWithTextVisible(reason)

    fun termsVisible(): Boolean {
        scrollToElementWithText(historyDetailScreenTermsTitle)
        return elementWithTextVisible(historyDetailScreenTermsTitle)
    }
}
