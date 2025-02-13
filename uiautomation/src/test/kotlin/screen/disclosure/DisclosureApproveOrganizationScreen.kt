package screen.disclosure

import util.MobileActions

class DisclosureApproveOrganizationScreen : MobileActions() {

    private val loginButton = find.byText(l10n.getString("organizationApprovePageLoginCta"))
    private val goToWebsiteButton = find.byText(l10n.getString("disclosureSuccessPageToWebsiteCta"))
    private val yesProceedButton = find.byText(l10n.getString("organizationApprovePageShareWithApproveCta"))
    private val shareButton = find.byText(l10n.getString("disclosureConfirmDataAttributesPageApproveCta"))
    private val closeButton = find.byText(l10n.getString("disclosureSuccessPageCloseCta"))
    private val attributesMissingMessage = find.byText(l10n.getString("disclosureMissingAttributesPageTitle"))

    fun loginButtonVisible() = isElementVisible(loginButton)

    fun login() = clickElement(loginButton)

    fun goToWebsite() {
        clickElement(goToWebsiteButton)
        switchToWebViewContext()
    }

    fun proceed() = clickElement(yesProceedButton)

    fun share() {
        scrollToEnd(ScrollableType.CustomScrollView)
        clickElement(shareButton)
    }

    fun close() {
        clickElement(closeButton)
        switchToWebViewContext()
    }

    fun attributesMissingMessageVisible() = isElementVisible(attributesMissingMessage, false)
}
