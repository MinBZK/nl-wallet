package screen.organization

import util.MobileActions

class OrganizationDetailScreen : MobileActions() {

    private val backButton = l10n.getString("generalBottomBackCta")

    private val legalNameLabel = l10n.getString("organizationDetailScreenLegalNameInfo")
    private val organizationTypeLabel = l10n.getString("organizationDetailScreenCategoryInfo")
    private val locationLabel = l10n.getString("organizationDetailScreenLocationInfo")
    private val organizationIdLabel = l10n.getString("organizationDetailScreenOrganizationIdInfo")

    fun organizationInHeaderVisible(organization: String) =
        elementWithTextVisible(l10n.getString("organizationDetailScreenTitle").replace("{name}", organization))

    fun descriptionVisible(description: String) = elementContainingTextVisible(description)

    fun legalNameVisible(legalName: String) = elementContainingTextWithSiblingVisible(legalName, legalNameLabel)

    fun organizationTypeVisible(publicBody: Boolean) = elementContainingTextWithSiblingVisible(
        l10n.getString(
            if (publicBody) {
                "organizationDetailScreenOrganizationTypeGovernment"
            } else {
                "organizationDetailScreenOrganizationTypeNonGovernment"
            }
        ),
        organizationTypeLabel,
    )

    fun organizationTypeNotVisible() = !elementWithTextVisible(organizationTypeLabel)

    fun locationVisible(country: String) = elementContainingTextWithSiblingVisible(country, locationLabel)

    fun organizationIdVisible(organizationId: String) =
        elementContainingTextWithSiblingVisible(organizationId, organizationIdLabel)

    private val websiteLabel = l10n.getString("organizationDetailScreenWebsiteInfo")
    private val supportLabel = l10n.getString("organizationDetailScreenSupportInfo")
    private val privacyLabel = l10n.getString("organizationDetailScreenPrivacyInfo")

    // The app shows the url without its scheme (UriDisplayFormatter), as the link's value next to the title label.
    private fun displayUri(uri: String) = uri.replace(Regex("^[a-zA-Z][a-zA-Z0-9+.-]*:(//)?"), "")

    fun websiteVisible(webUri: String) = elementWithLabelAndValueVisible(websiteLabel, displayUri(webUri))

    fun supportVisible(supportUri: String) = elementWithLabelAndValueVisible(supportLabel, displayUri(supportUri))

    fun privacyStatementVisible(privacyPolicyUri: String) =
        elementWithLabelAndValueVisible(privacyLabel, displayUri(privacyPolicyUri))

    fun clickBackButton() = clickElementWithText(backButton)
}
