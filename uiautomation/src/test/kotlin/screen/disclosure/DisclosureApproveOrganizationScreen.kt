package screen.disclosure

import domain.Platform
import util.MobileActions

class DisclosureApproveOrganizationScreen : MobileActions() {

    private val loginButton = l10n.getString("organizationApprovePageLoginCta")
    private val goToWebsiteButton = l10n.getString("disclosureSuccessPageToWebsiteCta")
    private val shareButton = l10n.getString("disclosureConfirmDataAttributesPageApproveCta")
    private val closeButton = l10n.getString("generalClose")
    private val viewLoginDisclosureDetailsButton = l10n.getString("organizationApprovePageMoreInfoLoginCta")
    private val viewDisclosureOrganizationDetailsButton = l10n.getString("requestDetailScreenAboutOrganizationCta")
    private val goBackButton = l10n.getString("generalBottomBackCta")
    private val stopRequestButton = l10n.getString("missingAttributesPageCloseCta")
    private val organizationApprovePageDenyCta = l10n.getString("organizationApprovePageDenyCta")
    private val disclosureStopSheetReportIssueCta = l10n.getString("disclosureStopSheetReportIssueCta")
    private val privacySectionTitle = l10n.getString("privacySectionTitle")
    private val privacySectionCta = l10n.getString("privacySectionCta")
    private val reportOptionSuspiciousOrganization = l10n.getString("reportOptionSuspiciousOrganization")
    private val swapCardButton = l10n.getString("sharedAttributesCardChangeCardCta")
    private val stopButton = l10n.getString("organizationApprovePageDenyCta")
    private val bottomSheetConfirmStopButton = l10n.getString("disclosureStopSheetPositiveCta")
    private val disclosureSuccessPageToDashboardCta = l10n.getString("disclosureSuccessPageToDashboardCta")
    private val toDashboardButton = l10n.getString("disclosureSuccessPageToDashboardCta")

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

    fun organizationNameForSharingFlowVisible(organizationName: String, timeoutInSeconds: Long = 5): Boolean {
        val selectorText = l10n.getString("disclosureConfirmDataAttributesShareWithTitle").replace("{organization}", organizationName)
        return elementWithTextVisible(selectorText, timeoutInSeconds);
    }

    fun organizationNameForLoginFlowVisible(organizationName: String): Boolean {
        val selectorText = l10n.getString("organizationApprovePageLoginTitle").replace("{organization}", organizationName)
        return elementWithTextVisible(selectorText);
    }

    fun viewDisclosureOrganizationDetails(organization: String) {
        clickElementContainingText(viewDisclosureOrganizationDetailsButton.replace("{organization}", organization))
    }


    fun viewLoginDisclosureDetails() = clickElementContainingText(viewLoginDisclosureDetailsButton)

    fun organizationDescriptionOnDetailsVisible(description: String): Boolean = elementWithTextVisible(description);

    fun goBack() = clickElementWithText(goBackButton)

    fun stopRequestAfterMissingAttributeFailure() = clickElementWithText(stopRequestButton)

    fun viewSharedData(count: String, cardTitle: String) {
        val title = l10n.getString("sharedAttributesCardTitle").replace("{count}", count).replace("{cardTitle}", cardTitle)
        clickElementContainingText(title)
    }

    fun bsnVisible(bsn: String) = elementContainingTextVisible(bsn)

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

    fun sharingReasonVisible(reason: String): Boolean {
        // TODO PVW-6101 check for purpose
        return true
    }

    fun privacyHeaderVisible(): Boolean {
        scrollToElementWithText(privacySectionTitle)
        return elementWithTextVisible(privacySectionTitle)
    }

    fun privacyButtonVisible(): Boolean  {
        scrollToElementWithText(privacySectionCta)
        return elementWithTextVisible(privacySectionCta)
    }

    fun clickSwapCardButton() {
        val element = scrollToElementWithText(swapCardButton)
        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        if (platform() == Platform.IOS) {
            // On iOS the accessibility frame of this button is shifted up relative
            // to its rendered position, so a tap at the reported center lands on a
            // non-interactive area. Probe downwards (never up, to avoid hitting the
            // card above) until the select-card sheet actually opens.
            val selectCardSheetTitle = l10n.getString("selectCardSheetTitle")
            val centerX = element.location.x + element.size.width / 2
            val centerY = element.location.y + element.size.height / 2
            for (offsetY in listOf(0, 30, 60, 90)) {
                tapAt(centerX, centerY + offsetY)
                if (elementWithTextVisible(selectCardSheetTitle, 2)) return
            }
            throw AssertionError("Select card sheet did not open after probing taps around '$swapCardButton'")
        } else {
            clickElementWithText(swapCardButton)
        }
    }

    fun swapCardTo(cardIdentifier: String) {
        clickElementContainingText(cardIdentifier)
    }

    fun stop() = clickElementWithText(stopButton)

    fun bottomSheetConfirmStop() = clickElementWithText(bottomSheetConfirmStopButton)

    fun goToDashboard() = clickElementWithText(toDashboardButton)

    fun goToDashBoard() =clickElementWithText(disclosureSuccessPageToDashboardCta)
}
