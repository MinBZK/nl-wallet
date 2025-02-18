package screen.history

import util.MobileActions

class HistoryDetailScreen : MobileActions() {

    private val screen = find.byValueKey("historyDetailScreen")

    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen, false)

    fun clickBottomBackButton() = clickElement(bottomBackButton, false)

    fun issuanceOrganizationVisible(organization: String): Boolean {
        return isElementVisible(find.byText(organization), false)
    }

    fun disclosureOrganizationVisible(organization: String): Boolean {
        val selector = find.byText(organization)
        return isElementVisible(selector, false)
    }

    fun titleCorrectForIssuance(card: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("historyDetailScreenTitleForIssuance").replace("{card}", card)), false)
    }


    fun titleCorrectForLogin(organization: String): Boolean {
        return isElementVisible(find.byText(l10n.getString("historyDetailScreenTitleForLogin").replace("{organization}", organization)), false)
    }
}
