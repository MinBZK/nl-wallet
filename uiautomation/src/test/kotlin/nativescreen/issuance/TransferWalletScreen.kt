package nativescreen.issuance

import util.NativeMobileActions

class TransferWalletScreen : NativeMobileActions() {

    private val createNewWalletButton = l10n.getString("walletTransferTargetScreenIntroductionOptOutCta")
    private val createNewWalletConfirmButton = l10n.getString("generalOkCta")

    fun createNewWallet() {
        clickElementWithText(createNewWalletButton)
        clickElementWithText(createNewWalletConfirmButton)
    }
}
