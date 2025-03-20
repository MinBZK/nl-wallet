package screen.history

import util.MobileActions

class HistoryOverviewScreen : MobileActions() {

    private val screen = find.byValueKey("historyOverviewScreen")

    private val pidCardTitle = find.byText(l10n.translate("NL Wallet persoonsgegevens"))
    private val addressCardTitle = find.byText(l10n.translate("NL Wallet adres"))
    private val disclosureLoginSubtitle = find.byText(l10n.getString("cardHistoryLoginSuccess"))

    private val bottomBackButton = find.byText(l10n.getString("generalBottomBackCta"))

    fun visible() = isElementVisible(screen, false)

    fun pidIssuanceLogEntryVisible() = isElementVisible(pidCardTitle, false)
    fun addressIssuanceLogEntryVisible() = isElementVisible(addressCardTitle, false)

    fun clickPidCardTitle() = clickElement(pidCardTitle, false)

    fun clickBottomBackButton() = clickElement(bottomBackButton, false)

    fun loginDisclosureLogEntryVisible() = isElementVisible(disclosureLoginSubtitle, false)

    fun clickLoginEntryTitle() = clickElement(disclosureLoginSubtitle)
}
