package nativescreen.card

import helper.OrganizationAuthMetadataHelper.Organization.RVIG
import util.NativeMobileActions

class CardDetailScreen : NativeMobileActions() {

    private val cardDetailScreenCardDataCta = l10n.getString("cardDetailScreenCardDataCta")
    private val pidIdTitleText = cardMetadata.getPidDisplayName()
    private val cardIssuerStateText = organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", RVIG)
    private val cardHistoryStateText = l10n.getString("cardDetailScreenLatestSuccessInteractionUnknown")
    private val cardDataButton = l10n.getString("cardDetailScreenCardDataCta")
    private val cardHistoryButton = l10n.getString("cardDetailScreenCardHistoryCta")
    private val bottomBackButton = l10n.getString("generalBottomBackCta")

    fun visible() = elementContainingTextVisible(cardDetailScreenCardDataCta)

    fun cardFaceElements() = elementContainingTextVisible(pidIdTitleText)

    fun issuerAndHistoryStates() = elementContainingTextVisible(cardIssuerStateText) && elementContainingTextVisible(cardHistoryStateText)

    fun clickCardDataButton() = clickElementContainingText(cardDataButton)

    fun clickCardHistoryButton() = clickElementContainingText(cardHistoryButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
