package screen.history

import util.MobileActions

class HistoryOverviewScreen : MobileActions() {

    private val screen = find.byValueKey("historyOverviewScreen")

    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen, false)

    fun clickBottomBackButton() = clickElement(bottomBackButton, false)
}
