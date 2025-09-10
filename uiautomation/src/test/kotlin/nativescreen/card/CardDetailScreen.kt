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

    fun visible() = elementWithTextVisible(cardDetailScreenCardDataCta)

    fun cardFaceElements() = elementWithTextVisible(pidIdTitleText)

    fun issuerAndHistoryStates() = elementWithTextVisible(cardIssuerStateText) && elementWithTextVisible(cardHistoryStateText)

    fun clickCardDataButton() = clickElementWithText(cardDataButton)

    fun clickCardHistoryButton() = clickElementWithText(cardHistoryButton)

    fun clickBottomBackButton() = clickElementWithText(bottomBackButton)
}
