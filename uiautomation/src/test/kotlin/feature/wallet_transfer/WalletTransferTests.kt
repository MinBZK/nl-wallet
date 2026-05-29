package feature.wallet_transfer

import helper.GbaDataHelper
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.NAME
import helper.TasDataHelper
import helper.TwoDeviceTestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataScreen
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import screen.error.NoInternetErrorScreen
import screen.issuance.StartTransferWalletScreen
import screen.security.PinScreen
import screen.wallet_transfer.WalletTransferSourceScreen
import screen.wallet_transfer.WalletTransferTargetScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("UC 9.10 Transfer Wallet")
@Tags(Tag("twoDevice"))
class WalletTransferTests : TwoDeviceTestBase() {

    private lateinit var gbaData: GbaDataHelper
    private lateinit var tasData: TasDataHelper

    // Source device
    private lateinit var sourceNavigator: OnboardingNavigator
    private lateinit var sourceDashboard: DashboardScreen
    private lateinit var sourceTransferScreen: WalletTransferSourceScreen
    private lateinit var sourcePin: PinScreen
    private lateinit var sourceNoInternetScreen: NoInternetErrorScreen

    // Destination device
    private lateinit var targetNavigator: OnboardingNavigator
    private lateinit var startTargetTransferScreen: StartTransferWalletScreen
    private lateinit var targetTransferScreen: WalletTransferTargetScreen
    private lateinit var targetDashboard: DashboardScreen
    private lateinit var targetCardDetailScreen: CardDetailScreen
    private lateinit var targetCardDataScreen: CardDataScreen

    fun setUp(testInfo: TestInfo) {
        startDrivers(testInfo)
        gbaData = GbaDataHelper()
        tasData = TasDataHelper()

        useSourceDevice {
            sourceNavigator = OnboardingNavigator()
            sourceDashboard = DashboardScreen()
            sourceTransferScreen = WalletTransferSourceScreen()
            sourcePin = PinScreen()
            sourceNoInternetScreen = NoInternetErrorScreen()
            sourceNavigator.toScreen(OnboardingNavigatorScreen.Dashboard)
        }

        useTargetDevice {
            targetNavigator = OnboardingNavigator()
            startTargetTransferScreen = StartTransferWalletScreen()
            targetTransferScreen = WalletTransferTargetScreen()
            targetDashboard = DashboardScreen()
            targetNavigator.toScreen(OnboardingNavigatorScreen.PersonalizeTransferWallet)
            startTargetTransferScreen.clickStartTransfer()
            targetCardDetailScreen = CardDetailScreen()
            targetCardDataScreen = CardDataScreen()
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC62 Happy flow: wallet is transferred to destination device")
    fun walletTransferHappyFlow(testInfo: TestInfo) {
        setUp(testInfo)
        var transferUrl = ""

        useTargetDevice {
            assertTrue(targetTransferScreen.qrScreenVisible(), "QR screen is not visible on destination device")
            transferUrl = targetTransferScreen.getTransferUrl()
        }

        useSourceDevice {
            sourceDashboard.openLink(transferUrl)
            assertTrue(sourceTransferScreen.confirmTransferVisible(), "Confirm transfer screen is not visible on source device")
            sourceTransferScreen.clickConfirmTransfer()
            sourcePin.enterPin(DEFAULT_PIN)
            assertTrue(sourceTransferScreen.transferringVisible(), "Transferring screen is not visible on source device")
        }

        useTargetDevice {
            assertTrue(targetTransferScreen.transferringVisible(), "Transferring screen is not visible on destination device")
            assertTrue(targetTransferScreen.successVisible(), "Success screen is not visible on destination device")
            targetTransferScreen.clickToOverview()
        }

        assertAll(
            { assertTrue(sourceDashboard.visible().not() || sourceTransferScreen.successVisible(), "Source device should show transfer success") },
            { assertTrue(targetDashboard.cardTitleVisible(), "Destination device dashboard should show the transferred PID card") },
        )

        useTargetDevice {
            targetDashboard.clickCard(tasData.getPidDisplayName())
            targetCardDetailScreen.clickCardDataButton()
            val nationalities = gbaData.getNationalities(DEFAULT_BSN)
            assertAll(
                { assertTrue(targetCardDataScreen.dataAttributeVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "data attribute are not visible") },
                { assertTrue(targetCardDataScreen.dataAttributeVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "data attribute are not visible") },
                { assertTrue(targetCardDataScreen.dataAttributeVisible(nationalities[0]), "array attribute is not visible") },
                { assertTrue(targetCardDataScreen.dataAttributeVisible(nationalities[1]), "array attribute is not visible") },
            )
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC63 Source device cancels transfer")
    fun walletTransferSourceLosesInternet(testInfo: TestInfo) {
        setUp(testInfo)
        var transferUrl = ""

        useTargetDevice {
            assertTrue(targetTransferScreen.qrScreenVisible(), "QR screen is not visible on destination device")
            transferUrl = targetTransferScreen.getTransferUrl()
        }

        useSourceDevice {
            sourceDashboard.openLink(transferUrl)
            assertTrue(sourceTransferScreen.confirmTransferVisible(), "Confirm transfer screen is not visible on source device")
            sourceTransferScreen.clickStop()
            sourceTransferScreen.confirmStop()
            assertTrue(sourceTransferScreen.stoppedVisible(), "Source device should show stopped")
        }

        useTargetDevice {
            assertTrue(targetTransferScreen.stoppedVisible(), "Target device should show stopped")
        }
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC64 Destination device cancels transfer")
    fun walletTransferDestinationCancels(testInfo: TestInfo) {
        setUp(testInfo)
        var transferUrl = ""

        useTargetDevice {
            assertTrue(targetTransferScreen.qrScreenVisible(), "QR screen is not visible on destination device")
            transferUrl = targetTransferScreen.getTransferUrl()
        }

        useSourceDevice {
            sourceDashboard.openLink(transferUrl)
            assertTrue(sourceTransferScreen.confirmTransferVisible(), "Confirm transfer screen is not visible on source device")
            sourceTransferScreen.clickConfirmTransfer()
        }

        useTargetDevice {
            targetTransferScreen.clickStop()
            targetTransferScreen.confirmStop()
            assertTrue(targetTransferScreen.stoppedVisible(), "Stopped screen is not visible on destination device")
        }

        useSourceDevice {
            assertTrue(sourceTransferScreen.stoppedVisible(), "Source device should show stopped")
        }
    }

}
