package feature.close_proximity

import helper.DeviceResponseHelper
import helper.OrganizationAuthMetadataHelper
import helper.TestBase
import navigator.MenuNavigator
import navigator.screen.MenuNavigatorScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.disclosure.BleDisconnectedScreen
import screen.disclosure.CloseProximityQrScreen
import screen.disclosure.DisclosureApproveOrganizationScreen
import screen.error.AttributesMissingErrorScreen
import screen.security.PinScreen
import util.captureOutput
import java.io.File

@DisplayName("Close Proximity Disclosure")
class CloseProximityDisclosureTests : TestBase() {

    companion object {
        private val READER_CA_CRT_FILE = System.getenv("READER_CA_CRT_FILE")
            ?: File("../scripts/devenv/target/ca.reader.crt.pem").canonicalPath
        private val READER_CA_KEY_FILE = System.getenv("READER_CA_KEY_FILE")
            ?: File("../scripts/devenv/target/ca.reader.key.pem").canonicalPath
    }

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var closeProximityQrScreen: CloseProximityQrScreen
    private lateinit var disclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var attributesMissingErrorScreen: AttributesMissingErrorScreen
    private lateinit var bleDisconnectedScreen: BleDisconnectedScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        dashboardScreen = DashboardScreen()
        closeProximityQrScreen = CloseProximityQrScreen()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        attributesMissingErrorScreen = AttributesMissingErrorScreen()
        bleDisconnectedScreen = BleDisconnectedScreen()
    }

    @RetryingTest(value = 2, name = "{displayName} - {index}")
    @DisplayName("LTC79 Close proximity data sharing")
    fun verifyCloseProximityDisclosureViaQrScan(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
        dashboardScreen.showQRCode()
        closeProximityQrScreen.centerQr()
        val qrString = closeProximityQrScreen.getQr()
        val mockBleReaderApp = closeProximityQrScreen.startMockBleReaderApp(
            qrString,
            readerCaCrtFile = READER_CA_CRT_FILE,
            readerCaKeyFile = READER_CA_KEY_FILE,
            readerAuthFile = File("../scripts/devenv/mijn_amsterdam_reader_auth.json").canonicalPath,
            waitForDeviceResponse = true,
        )
        val outputBuffer = mockBleReaderApp.captureOutput()

        disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName",
            OrganizationAuthMetadataHelper.Organization.AMSTERDAM
        ))
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToDashBoard()
        val exitCode = mockBleReaderApp.waitFor()
        val hex = DeviceResponseHelper.extractHex(outputBuffer.toString())
        val deviceResponse = DeviceResponseHelper.parse(hex!!)
        val doc = deviceResponse.documents.first()
        val bsn = doc.attributes.firstOrNull { it.identifier == "bsn" }?.value

        assertAll(
            { assertTrue(dashboardScreen.visible(), "Dashboard is not visible") },
            { assertTrue(exitCode == 0, "Mac reader failed (exit $exitCode):\n$outputBuffer" ) },
            { assertTrue(deviceResponse.version == "1.0", "Device response version mismatch") },
            { assertTrue(deviceResponse.status == 0, "Device response status is not success") },
            { assertTrue(deviceResponse.documents.size == 1, "Expected exactly one document") },
            { assertTrue(doc.docType == "urn:eudi:pid:nl:1", "Document type mismatch") },
            { assertTrue(bsn == DEFAULT_BSN, "BSN attribute mismatch") },
        )
    }

    @RetryingTest(value = 2, name = "{displayName} - {index}")
    @DisplayName("LTC80 Wallet does not contain requested attributes at close proximity disclosure")
    fun verifyCloseProximityWalletDoesNotContainRequestedAttributes(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
        dashboardScreen.showQRCode()
        closeProximityQrScreen.centerQr()
        val qrString = closeProximityQrScreen.getQr()
        closeProximityQrScreen.startMockBleReaderApp(
            qrString,
            readerCaCrtFile = READER_CA_CRT_FILE,
            readerCaKeyFile = READER_CA_KEY_FILE,
            readerAuthFile = File("../scripts/devenv/monkey_bike_reader_auth.json").canonicalPath,
        )
        assertTrue(attributesMissingErrorScreen.attributesMissingMessageVisible(), "Attributes missing message not visible")
    }

    @RetryingTest(value = 2, name = "{displayName} - {index}")
    @DisplayName("LTC81 BLE connection lost from reader during close proximity disclosure")
    fun verifyCloseProximityBLEDisconnect(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
        dashboardScreen.showQRCode()
        closeProximityQrScreen.centerQr()
        val qrString = closeProximityQrScreen.getQr()
        val mockBleReaderApp = closeProximityQrScreen.startMockBleReaderApp(
            qrString,
            readerCaCrtFile = READER_CA_CRT_FILE,
            readerCaKeyFile = READER_CA_KEY_FILE,
            readerAuthFile = File("../scripts/devenv/mijn_amsterdam_reader_auth.json").canonicalPath,
            waitForDeviceResponse = true,
        )

        disclosureScreen.organizationNameForSharingFlowVisible(organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName",
            OrganizationAuthMetadataHelper.Organization.AMSTERDAM
        ))
        disclosureScreen.share()

        mockBleReaderApp.destroyForcibly()
        pinScreen.enterPin(DEFAULT_PIN)
        assertTrue(bleDisconnectedScreen.visible(), "BLE disconnected screen is not visible")
    }
}
