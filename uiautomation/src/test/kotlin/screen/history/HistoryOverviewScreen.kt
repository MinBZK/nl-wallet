package screen.history

import util.MobileActions

class HistoryOverviewScreen : MobileActions() {

    private val pidCardTitle = cardMetadata.getPidDisplayName()
    private val addressCardTitle = cardMetadata.getAddressDisplayName()
    private val disclosureLoginSubtitle = l10n.getString("cardHistoryLoginSuccess")
    private val historyDetailScreenIssuanceSuccessDescription= l10n.getString("historyDetailScreenIssuanceSuccessDescription")
    private val cardHistoryTimelineOperationRenewed = l10n.getString("cardHistoryTimelineOperationRenewed")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementWithTextVisible(bottomBackButton)

    fun pidIssuanceLogEntryVisible() = elementContainingTextVisible(pidCardTitle)

    fun addressIssuanceLogEntryVisible() = elementContainingTextVisible(addressCardTitle)

    fun clickPidCardTitle() = clickElementContainingText(pidCardTitle)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)

    fun loginDisclosureLogEntryVisible() = elementContainingTextVisible(disclosureLoginSubtitle)

    fun clickLoginEntryTitle() = clickElementContainingText(disclosureLoginSubtitle)

    fun disclosureOrganizationVisible(organizatioName: String) = elementContainingTextVisible(organizatioName)

    fun issuanceSubtitleVisible() = elementContainingTextVisible(historyDetailScreenIssuanceSuccessDescription)

    fun renewCardSubtitleVisible() = elementContainingTextVisible(cardHistoryTimelineOperationRenewed)
}
