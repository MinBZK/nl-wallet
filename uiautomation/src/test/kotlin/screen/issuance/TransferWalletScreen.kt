package screen.issuance

import util.MobileActions

class TransferWalletScreen : MobileActions() {

    private val createNewWalletButton = l10n.getString("walletTransferTargetScreenIntroductionOptOutCta")
    private val createNewWalletConfirmButton = l10n.getString("generalOkCta")

    fun createNewWallet() {
        clickElementWithText(createNewWalletButton)
        clickElementWithText(createNewWalletConfirmButton)
    }
}
