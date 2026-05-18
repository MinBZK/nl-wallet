package screen.wallet_transfer

import com.google.zxing.BinaryBitmap
import com.google.zxing.MultiFormatReader
import com.google.zxing.client.j2se.BufferedImageLuminanceSource
import com.google.zxing.common.HybridBinarizer
import org.openqa.selenium.OutputType
import org.openqa.selenium.TakesScreenshot
import util.MobileActions
import java.io.ByteArrayInputStream
import javax.imageio.ImageIO

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

    fun getTransferUrl(): String {
        val screenshotBytes = (driver as TakesScreenshot).getScreenshotAs(OutputType.BYTES)
        val image = ImageIO.read(ByteArrayInputStream(screenshotBytes))
        val source = BufferedImageLuminanceSource(image)
        val binaryBitmap = BinaryBitmap(HybridBinarizer(source))
        return MultiFormatReader().decodeWithState(binaryBitmap).text
    }

    fun awaitingConfirmationVisible() = elementWithTextVisible(awaitingConfirmationTitle)

    fun clickStop() = clickElementWithText(stopCta)

    fun confirmStop() = clickElementWithText(stopSheetConfirmCta)

    fun transferringVisible(timeoutSeconds: Long = 30) = elementWithTextVisible(transferringTitle, timeoutSeconds)

    fun successVisible(timeoutSeconds: Long = 180) = elementWithTextVisible(successTitle, timeoutSeconds)

    fun clickToOverview() = clickElementWithText(toOverviewCta)

    fun stoppedVisible() = elementWithTextVisible(stoppedTitle, 15)
}
