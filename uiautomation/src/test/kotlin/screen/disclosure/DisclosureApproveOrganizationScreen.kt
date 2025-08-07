package screen.disclosure

import util.MobileActions

class DisclosureApproveOrganizationScreen : MobileActions() {

    private val loginButton = find.byText(l10n.getString("organizationApprovePageLoginCta"))
    private val goToWebsiteButton = find.byText(l10n.getString("disclosureSuccessPageToWebsiteCta"))
    private val yesProceedButton = find.byText(l10n.getString("disclosureConfirmDataAttributesPageApproveCta"))
    private val shareButton = find.byText(l10n.getString("disclosureConfirmDataAttributesPageApproveCta"))
    private val closeButton = find.byText(l10n.getString("disclosureSuccessPageCloseCta"))
    private val attributesMissingMessage = find.byText(l10n.getString("missingAttributesPageTitle"))
    private val viewActivitiesButton = find.byText(l10n.getString("disclosureSuccessPageShowHistoryCta"))
    private val viewLoginDisclosureDetailsButton = find.byText(l10n.getString("organizationApprovePageMoreInfoLoginCta"))
    private val viewDisclosureOrganizationDetailsButton = find.byText(l10n.getString("organizationButtonLabel"))
    private val goBackButton = find.byText(l10n.getString("generalBottomBackCta"))
    private val closeDialogButton = find.byValueKey("close_icon_button")
    private val stopRequestButton = find.byText(l10n.getString("missingAttributesPageCloseCta"))
    private val readTermsButton = find.byText(l10n.getString("loginDetailScreenAgreementCta"))
    private val termsSubtitle = find.byText(l10n.getString("policyScreenSubtitle"))
    private val organizationApprovePageDenyCta = find.byText(l10n.getString("organizationApprovePageDenyCta"))
    private val disclosureStopSheetReportIssueCta = find.byText(l10n.getString("disclosureStopSheetReportIssueCta"))
    private val disclosureConfirmDataAttributesSubtitleTerms = find.byText(l10n.getString("disclosureConfirmDataAttributesSubtitleTerms"))
    private val disclosureConfirmDataAttributesCheckConditionsCta = find.byText(l10n.getString("disclosureConfirmDataAttributesCheckConditionsCta"))
    private val reportOptionUntrusted = find.byText(l10n.getString("reportOptionUntrusted"))
    private val reportOptionSuspiciousOrganization = find.byText(l10n.getString("reportOptionSuspiciousOrganization"))

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
        val selector = l10n.getString("disclosureConfirmDataAttributesShareWithTitle").replace("{organization}", organizationName)
        val element = find.byText(selector)
        return isElementVisible(element);
    }

    fun organizationNameForLoginFlowVisible(organizationName: String): Boolean {
        val selector = l10n.getString("organizationApprovePageLoginTitle").replace("{organization}", organizationName)
        val element = find.byText(selector)
        return isElementVisible(element);
    }

    fun viewDisclosureOrganizationDetails() {
        clickElement(viewDisclosureOrganizationDetailsButton)
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

    fun viewSharedData(count: String, cardTitle: String) {
        val title = l10n.getString("sharedAttributesCardTitle").replace("{count}", count).replace("{cardTitle}", cardTitle)
        clickElementContainingText(title)
    }

    fun bsnVisible(bsn: String): Boolean {
        return isElementVisible(find.byText(bsn))
    }

    fun readTerms() {
        scrollToEnd(ScrollableType.CustomScrollView)
        clickElement(readTermsButton)
    }

    fun termsVisible(): Boolean {
        return isElementVisible(termsSubtitle)
    }

    fun viewOrganization(organization: String) {
        clickElement(find.byText(organization))
    }

    fun cancel() {
        scrollToEnd(ScrollableType.CustomScrollView)
        clickElement(organizationApprovePageDenyCta)
    }

    fun reportProblem() = clickElement(disclosureStopSheetReportIssueCta)

    fun reportOptionUntrustedVisible() = isElementVisible(reportOptionUntrusted)

    fun reportOptionSuspiciousVisible() = isElementVisible(reportOptionSuspiciousOrganization)


    fun organizationInPresentationRequestHeaderVisible(organization: String): Boolean {
        val selector = l10n.getString("disclosureConfirmDataAttributesShareWithTitle").replace("{organization}", organization)
        val element = find.byText(selector)
        return isElementVisible(element);
    }

    fun labelVisible(label: String): Boolean {
        return isElementVisible(find.byText(label))
    }

    fun dataNotVisible(data: String): Boolean {
        return !isElementVisible(find.byText(data))
    }

    fun dataVisible(data: String): Boolean {
        return isElementVisible(find.byText(data))
    }

    fun sharingReasonVisible(reason: String): Boolean {
        return isElementVisible(find.byText(reason))
    }

    fun conditionsHeaderVisible(): Boolean {
        scrollToEnd(ScrollableType.CustomScrollView)
        return isElementVisible(disclosureConfirmDataAttributesSubtitleTerms)
    }

    fun conditionsButtonVisible(): Boolean  {
        scrollToEnd(ScrollableType.CustomScrollView)
        return isElementVisible(disclosureConfirmDataAttributesCheckConditionsCta)
    }
}
