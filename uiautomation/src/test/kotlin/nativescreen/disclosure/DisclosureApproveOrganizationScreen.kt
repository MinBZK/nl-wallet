package nativescreen.disclosure

import util.NativeMobileActions

class DisclosureApproveOrganizationScreen : NativeMobileActions() {

    private val loginButton = l10n.getString("organizationApprovePageLoginCta")
    private val goToWebsiteButton = l10n.getString("disclosureSuccessPageToWebsiteCta")
    private val shareButton = l10n.getString("disclosureConfirmDataAttributesPageApproveCta")
    private val closeButton = l10n.getString("disclosureSuccessPageCloseCta")
    private val attributesMissingMessage = l10n.getString("missingAttributesPageTitle")
    private val viewLoginDisclosureDetailsButton = l10n.getString("organizationApprovePageMoreInfoLoginCta")
    private val viewDisclosureOrganizationDetailsButton = l10n.getString("organizationButtonLabel")
    private val goBackButton = l10n.getString("generalBottomBackCta")
    private val stopRequestButton = l10n.getString("missingAttributesPageCloseCta")
    private val readTermsButton = l10n.getString("loginDetailScreenAgreementCta")
    private val termsSubtitle = l10n.getString("policyScreenSubtitle")
    private val organizationApprovePageDenyCta = l10n.getString("organizationApprovePageDenyCta")
    private val disclosureStopSheetReportIssueCta = l10n.getString("disclosureStopSheetReportIssueCta")
    private val disclosureConfirmDataAttributesSubtitleTerms = l10n.getString("disclosureConfirmDataAttributesSubtitleTerms")
    private val disclosureConfirmDataAttributesCheckConditionsCta = l10n.getString("disclosureConfirmDataAttributesCheckConditionsCta")
    private val reportOptionSuspiciousOrganization = l10n.getString("reportOptionSuspiciousOrganization")

    fun login() = clickElementWithText(loginButton)

    fun goToWebsite() {
        clickElementWithText(goToWebsiteButton)
        switchToWebViewContext()
    }

    fun share() {
        scrollToElementWithText(shareButton)
        clickElementWithText(shareButton)
    }

    fun close() {
        clickElementWithText(closeButton)
        switchToWebViewContext()
    }

    fun attributesMissingMessageVisible() = elementWithTextVisible(attributesMissingMessage)

    fun organizationNameForSharingFlowVisible(organizationName: String): Boolean {
        val selectorText = l10n.getString("disclosureConfirmDataAttributesShareWithTitle").replace("{organization}", organizationName)
        return elementWithTextVisible(selectorText);
    }

    fun organizationNameForLoginFlowVisible(organizationName: String): Boolean {
        val selectorText = l10n.getString("organizationApprovePageLoginTitle").replace("{organization}", organizationName)
        return elementWithTextVisible(selectorText);
    }

    fun viewDisclosureOrganizationDetails() = clickElementContainingText(viewDisclosureOrganizationDetailsButton)

    fun viewLoginDisclosureDetails() = clickElementContainingText(viewLoginDisclosureDetailsButton)

    fun organizationDescriptionOnDetailsVisible(description: String): Boolean = elementWithTextVisible(description);

    fun goBack() = clickElementWithText(goBackButton)

    fun stopRequestAfterMissingAttributeFailure() = clickElementWithText(stopRequestButton)

    fun closeDisclosureAfterCompletedOrUncompleted() = clickElementWithText(closeButton)

    fun viewSharedData(count: String, cardTitle: String) {
        val title = l10n.getString("sharedAttributesCardTitle").replace("{count}", count).replace("{cardTitle}", cardTitle)
        clickElementContainingText(title)
    }

    fun bsnVisible(bsn: String) = elementContainingTextVisible(bsn)

    fun readTerms() {
        scrollToElementWithText(readTermsButton)
        clickElementWithText(readTermsButton)
    }

    fun termsVisible() = elementWithTextVisible(termsSubtitle)

    fun viewOrganization(organization: String) = clickElementContainingText(organization)

    fun cancel() {
        scrollToElementWithText(organizationApprovePageDenyCta)
        clickElementWithText(organizationApprovePageDenyCta)
    }

    fun reportProblem() = clickElementWithText(disclosureStopSheetReportIssueCta)

    fun reportOptionSuspiciousVisible() = elementWithTextVisible(reportOptionSuspiciousOrganization)

    fun organizationInPresentationRequestHeaderVisible(organization: String): Boolean {
        val selectorText = l10n.getString("disclosureConfirmDataAttributesShareWithTitle").replace("{organization}", organization)
        return elementWithTextVisible(selectorText);
    }

    fun labelVisible(label: String) = elementContainingTextVisible(label)

    fun dataNotVisible(data: String) = !elementContainingTextVisible(data)

    fun dataVisible(data: String) = elementContainingTextVisible(data)

    fun sharingReasonVisible(reason: String) = elementWithTextVisible(reason)

    fun conditionsHeaderVisible(): Boolean {
        scrollToElementWithText(disclosureConfirmDataAttributesSubtitleTerms)
        return elementWithTextVisible(disclosureConfirmDataAttributesSubtitleTerms)
    }

    fun conditionsButtonVisible(): Boolean  {
        scrollToElementWithText(disclosureConfirmDataAttributesCheckConditionsCta)
        return elementWithTextVisible(disclosureConfirmDataAttributesCheckConditionsCta)
    }
}
