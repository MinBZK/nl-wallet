package screen.disclosure

import util.MobileActions

class DisclosureApproveOrganizationScreen : MobileActions() {

    private val loginButton = find.byText(l10n.getString("organizationApprovePageLoginCta"))
    private val goToWebsiteButton = find.byText(l10n.getString("disclosureSuccessPageToWebsiteCta"))
    private val yesProceedButton = find.byText(l10n.getString("organizationApprovePageShareWithApproveCta"))
    private val shareButton = find.byText(l10n.getString("disclosureConfirmDataAttributesPageApproveCta"))
    private val closeButton = find.byText(l10n.getString("disclosureSuccessPageCloseCta"))
    private val attributesMissingMessage = find.byText(l10n.getString("disclosureMissingAttributesPageTitle"))
    private val viewActivitiesButton = find.byText(l10n.getString("disclosureSuccessPageShowHistoryCta"))
    private val viewLoginDisclosureDetailsButton = find.byText(l10n.getString("organizationApprovePageMoreInfoLoginCta"))
    private val viewDisclosureDetailsButton = find.byText(l10n.getString("organizationApprovePageMoreInfoCta"))
    private val goBackButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val closeDialogButton = find.byValueKey("close_icon_button")
    private val stopRequestButton = find.byText(l10n.getString("disclosureMissingAttributesPageCloseCta"))
    private val viewSharedDataButton = find.byText(l10n.getString("sharedAttributesCardCta"))
    private val readTermsButton = find.byText(l10n.getString("loginDetailScreenAgreementCta"))
    private val termsSubtitle = find.byText(l10n.getString("policyScreenSubtitle"))


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

    fun viewActivities() {
        clickElement(viewActivitiesButton)
    }

    fun organizationNameForSharingFlowVisible(organizationName: String): Boolean {
        val selector = l10n.getString("organizationApprovePageGenericTitle").replace("{organization}", organizationName)
        val element = find.byText(selector)
        return isElementVisible(element);
    }

    fun organizationNameForLoginFlowVisible(organizationName: String): Boolean {
        val selector = l10n.getString("organizationApprovePageLoginTitle").replace("{organization}", organizationName)
        val element = find.byText(selector)
        return isElementVisible(element);
    }

    fun viewDisclosureDetails() {
        clickElement(viewDisclosureDetailsButton)
    }

    fun viewLoginDisclosureDetails() {
        clickElement(viewLoginDisclosureDetailsButton)
    }

    fun organizationDescriptionOnDetailsVisible(description: String): Boolean {
        val element = find.byText(description)
        return isElementVisible(element);
    }

    fun goBack() {
        clickElement(goBackButton)
    }

    fun closeDialog() {
        clickElement(closeDialogButton)
    }

    fun stopRequestAfterMissingAttributeFailure() {
        clickElement(stopRequestButton)
    }

    fun closeDisclosureAfterCompletedOrUncompleted() {
        clickElement(closeButton)
    }

    fun viewSharedData() {
        clickElement(viewSharedDataButton)
    }

    fun bsnVisible(bsn: String): Boolean {
        return isElementVisible(find.byText(bsn))
    }

    fun readTerms() {
        clickElement(readTermsButton)
    }

    fun termsVisible(): Boolean {
        return isElementVisible(termsSubtitle)
    }
}

