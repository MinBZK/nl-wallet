package screen.organization

import util.MobileActions

class OrganizationDetailScreen : MobileActions() {

    private val backButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun organizationInHeaderVisible(organization: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("organizationDetailScreenTitle").replace("{name}", organization)))
    }

    fun clickBackButton() {
        clickElement(backButton)
    }
}
