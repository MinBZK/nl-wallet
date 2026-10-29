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

    fun websiteVisible(webUri: String) = elementContainingTextVisible(webUri)

    fun supportVisible(supportUri: String) = elementContainingTextVisible(supportUri)

    fun privacyStatementVisible(privacyPolicyUri: String) = elementContainingTextVisible(privacyPolicyUri)

    fun clickBackButton() = clickElementWithText(backButton)
}
