package screen.wallet_transfer

import org.openqa.selenium.OutputType
import org.openqa.selenium.TakesScreenshot
import util.MobileActions

class WalletTransferTargetScreen : MobileActions() {

    private val qrScreenTitle = l10n.getString("walletTransferAwaitingScanPageTitle")
    private val awaitingConfirmationTitle = l10n.getString("walletTransferAwaitingConfirmationPageTitle")
    private val stopCta = l10n.getString("walletTransferAwaitingConfirmationPageCta")
    private val transferringTitle = l10n.getString("walletTransferTargetScreenTransferringTitle")
    private val successTitle = l10n.getString("walletTransferTargetScreenSuccessTitle")
    private val toOverviewCta = l10n.getString("walletTransferTargetScreenSuccessCta")
    private val stopSheetConfirmCta = l10n.getString("walletTransferTargetStopSheetConfirmCta")
    private val stoppedTitle = l10n.getString("walletTransferScreenStoppedTitle")

    fun qrScreenVisible() = elementWithTextVisible(qrScreenTitle)

    fun getTransferUrl(): String =
        decodeQrFromBytes((driver as TakesScreenshot).getScreenshotAs(OutputType.BYTES))

    fun awaitingConfirmationVisible() = elementWithTextVisible(awaitingConfirmationTitle)

    fun clickStop() = clickElementWithText(stopCta)

    fun confirmStop() = clickElementWithText(stopSheetConfirmCta)

    fun transferringVisible(timeoutSeconds: Long = 30) = elementWithTextVisible(transferringTitle, timeoutSeconds)

    fun successVisible(timeoutSeconds: Long = 180) = elementWithTextVisible(successTitle, timeoutSeconds)

    fun clickToOverview() = clickElementWithText(toOverviewCta)

    fun stoppedVisible() = elementWithTextVisible(stoppedTitle, 15)
}
