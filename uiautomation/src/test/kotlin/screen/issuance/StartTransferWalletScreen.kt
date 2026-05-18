package screen.issuance

import util.MobileActions

class StartTransferWalletScreen : MobileActions() {

    private val createNewWalletButton = l10n.getString("walletTransferTargetScreenIntroductionOptOutCta")
    private val createNewWalletConfirmButton = l10n.getString("generalOkCta")
    private val startTransferButton = l10n.getString("walletTransferTargetScreenIntroductionOptInCta")

    fun createNewWallet() {
        clickElementWithText(createNewWalletButton, 12)
        clickElementWithText(createNewWalletConfirmButton, 12)
    }

    fun clickStartTransfer() = clickElementWithText(startTransferButton, 12)

}
