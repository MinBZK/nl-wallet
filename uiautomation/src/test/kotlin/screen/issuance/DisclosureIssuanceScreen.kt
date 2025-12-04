package screen.issuance

import util.MobileActions

class DisclosureIssuanceScreen : MobileActions() {

    private val viewDetailsButton = l10n.getString("organizationApprovePageMoreInfoIssuanceCta")
    private val goBackButton = l10n.getString("generalBottomBackCta")
    private val shareButton = l10n.getString("disclosureConfirmDataAttributesPageApproveCta")
    private val stopButton = l10n.getString("organizationApprovePageDenyCta")
    private val bottomSheetConfirmStopButton = l10n.getString("disclosureStopSheetPositiveCta")

    fun organizationNameVisible(organizationName: String): Boolean {
        val selector = l10n.getString("organizationApprovePageIssuanceTitle").replace("{organization}", organizationName)
        return elementWithTextVisible(selector);
    }

    fun viewDetails() = clickElementWithText(viewDetailsButton)

    fun requestedAttributeVisible(attribute: String) = elementContainingTextVisible(attribute)

    fun goBack() = clickElementWithText(goBackButton)

    fun share() = clickElementWithText(shareButton)

    fun stop() = clickElementWithText(stopButton)

    fun bottomSheetConfirmStop() = clickElementWithText(bottomSheetConfirmStopButton)
}
