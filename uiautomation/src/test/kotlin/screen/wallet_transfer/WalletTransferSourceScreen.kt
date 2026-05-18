package screen.wallet_transfer

import util.MobileActions

class WalletTransferSourceScreen : MobileActions() {

    private val confirmTransferTitle = l10n.getString("walletTransferSourceScreenIntroductionTitle")
    private val confirmTransferCta = l10n.getString("walletTransferSourceScreenIntroductionCta")
    private val transferringTitle = l10n.getString("walletTransferSourceScreenTransferringTitle")
    private val successTitle = l10n.getString("walletTransferSourceScreenSuccessTitle")
    private val stoppedTitle = l10n.getString("walletTransferScreenStoppedTitle")
    private val stopSheetConfirmCta = l10n.getString("walletTransferSourceStopSheetConfirmCta")
    private val stopCta = l10n.getString("walletTransferAwaitingConfirmationPageCta")

    fun confirmTransferVisible() = elementWithTextVisible(confirmTransferTitle)

    fun clickConfirmTransfer() = clickElementWithText(confirmTransferCta)

    fun transferringVisible(timeoutSeconds: Long = 30) = elementWithTextVisible(transferringTitle, timeoutSeconds)

    fun successVisible(timeoutSeconds: Long = 180) = elementWithTextVisible(successTitle, timeoutSeconds)

    fun stoppedVisible() = elementWithTextVisible(stoppedTitle)

    fun clickStop() = clickElementWithText(stopCta)

    fun confirmStop() = clickElementWithText(stopSheetConfirmCta)
}
