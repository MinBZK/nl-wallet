package screen.card

import helper.OrganizationAuthMetadataHelper.Organization.RVIG
import util.MobileActions

class CardDetailScreen : MobileActions() {

    private val screen = find.byValueKey("cardDetailScreen")

    private val pidIdTitleText = find.byText(cardMetadata.getPidDisplayName())
    private val cardIssuerStateText = find.byText(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", RVIG))
    private val cardHistoryStateText = find.byText(l10n.getString("cardDetailScreenLatestSuccessInteractionUnknown"))

    private val cardDataButton = find.byText(l10n.getString("cardDetailScreenCardDataCta"))
    private val cardHistoryButton = find.byText(l10n.getString("cardDetailScreenCardHistoryCta"))
    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen, false)

    fun cardFaceElements() = isElementVisible(pidIdTitleText, false)

    fun issuerAndHistoryStates() =
        isElementVisible(cardIssuerStateText, false) && isElementVisible(cardHistoryStateText, false)

    fun clickCardDataButton() = clickElement(cardDataButton, false)

    fun clickCardHistoryButton() = clickElement(cardHistoryButton, false)

    fun clickBottomBackButton() = clickElement(bottomBackButton, false)
}
