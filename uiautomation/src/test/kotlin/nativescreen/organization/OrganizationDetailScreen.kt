package nativescreen.organization

import util.NativeMobileActions

class OrganizationDetailScreen : NativeMobileActions() {

    private val backButton = l10n.getString("generalBottomBackCta")

    fun organizationInHeaderVisible(organization: String) =
        elementWithTextVisible(l10n.getString("organizationDetailScreenTitle").replace("{name}", organization))

    fun clickBackButton() = clickElementWithText(backButton)
}
