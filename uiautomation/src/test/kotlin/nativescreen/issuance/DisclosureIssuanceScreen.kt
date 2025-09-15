package nativescreen.issuance

import util.NativeMobileActions

class DisclosureIssuanceScreen : NativeMobileActions() {

    private val viewDetailsButton = l10n.getString("organizationApprovePageMoreInfoIssuanceCta")
    private val goBackButton = l10n.getString("generalBottomBackCta")
    private val shareButton = l10n.getString("disclosureConfirmDataAttributesPageApproveCta")

    fun organizationNameVisible(organizationName: String): Boolean {
        val selector = l10n.getString("organizationApprovePageIssuanceTitle").replace("{organization}", organizationName)
        return elementWithTextVisible(selector);
    }

    fun viewDetails() = clickElementWithText(viewDetailsButton)

    fun requestedAttributeVisible(attribute: String): Boolean = elementContainingTextVisible(attribute)

    fun goBack() = clickElementWithText(goBackButton)

    fun share() = clickElementWithText(shareButton)
}
