package screen.history

import util.MobileActions

class HistoryDetailScreen : MobileActions() {

    private val screen = find.byValueKey("historyDetailScreen")

    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen, false)

    fun clickBottomBackButton() = clickElement(bottomBackButton, false)
}
