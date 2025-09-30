package screen.organization

import util.MobileActions

class OrganizationDetailScreen : MobileActions() {

    private val backButton = l10n.getString("generalBottomBackCta")

    fun organizationInHeaderVisible(organization: String) =
        elementWithTextVisible(l10n.getString("organizationDetailScreenTitle").replace("{name}", organization))

    fun clickBackButton() = clickElementWithText(backButton)
}
