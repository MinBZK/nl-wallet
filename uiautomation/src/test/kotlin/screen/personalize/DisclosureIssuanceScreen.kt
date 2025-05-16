package screen.personalize

import util.MobileActions

class DisclosureIssuanceScreen : MobileActions() {

    private val viewDetailsButton = find.byText(l10n.getString("organizationApprovePageMoreInfoIssuanceCta"))
    private val goBackButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val shareButton = find.byText(l10n.getString("disclosureConfirmDataAttributesPageApproveCta"))

    fun organizationNameVisible(organizationName: String): Boolean {
        val selector = l10n.getString("organizationApprovePageIssuanceTitle").replace("{organization}", organizationName)
        val element = find.byText(selector)
        return isElementVisible(element);
    }

    fun viewDetails() {
        clickElement(viewDetailsButton)
    }

    fun requestedAttributeVisible(attribute: String): Boolean {
        val keyElement = find.byText(attribute)
        return isElementVisible(keyElement)
    }

    fun goBack() {
        clickElement(goBackButton)
    }

    fun share() {
        clickElement(shareButton)
    }

}
