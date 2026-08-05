package feature.close_proximity

import domain.Platform
import helper.DeviceResponseHelper
import helper.OrganizationMetadataHelper
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
        private val AMSTERDAM_CA_CRT_FILE = System.getenv("AMSTERDAM_CA_CRT_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/mijn_amsterdam.crt.pem").canonicalPath
        private val AMSTERDAM_CA_KEY_FILE = System.getenv("AMSTERDAM_CA_KEY_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/mijn_amsterdam.key.pem").canonicalPath
        private val MONKEY_BIKE_CA_CRT_FILE = System.getenv("MONKEY_BIKE_CA_CRT_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/monkey_bike.crt.pem").canonicalPath
        private val MONKEY_BIKE_CA_KEY_FILE = System.getenv("MONKEY_BIKE_CA_KEY_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/monkey_bike.key.pem").canonicalPath

        private const val PID_DOC_TYPE = "urn:eudi:pid:nl:1"
        private val AMSTERDAM_ATTRIBUTES = listOf("urn:eudi:pid:nl:1/bsn")
        private val MONKEY_BIKE_ATTRIBUTES = listOf(
            "urn:eudi:pid:nl:1/given_name",
            "urn:eudi:pid:nl:1/family_name",
            "urn:eudi:pid:nl:1/birthdate",
            "urn:eudi:pid:nl:1/gender",
            "urn:eudi:pid:nl:1.address/street_address",
            "urn:eudi:pid:nl:1.address/house_number",
            "urn:eudi:pid:nl:1.address/postal_code",
            "urn:eudi:pid:nl:1.address/locality",
        )
        private const val READER_STARTUP_TIMEOUT_SECONDS = 40L
    }

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var closeProximityQrScreen: CloseProximityQrScreen
    private lateinit var disclosureScreen: DisclosureApproveOrganizationScreen
    private lateinit var pinScreen: PinScreen
    private lateinit var organizationAuthMetadata: OrganizationMetadataHelper
    private lateinit var attributesMissingErrorScreen: AttributesMissingErrorScreen
    private lateinit var bleDisconnectedScreen: BleDisconnectedScreen

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        dashboardScreen = DashboardScreen()
        closeProximityQrScreen = CloseProximityQrScreen()
        disclosureScreen = DisclosureApproveOrganizationScreen()
        pinScreen = PinScreen()
        organizationAuthMetadata = OrganizationMetadataHelper()
        attributesMissingErrorScreen = AttributesMissingErrorScreen()
        bleDisconnectedScreen = BleDisconnectedScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
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
            wrpacCaCrtFile = AMSTERDAM_CA_CRT_FILE,
            wrpacCaKeyFile = AMSTERDAM_CA_KEY_FILE,
            requestDocType = PID_DOC_TYPE,
            requestAttributes = AMSTERDAM_ATTRIBUTES,
            waitForDeviceResponse = true,
        )
        val outputBuffer = mockBleReaderApp.captureOutput()

        assertTrue(
            disclosureScreen.organizationNameForSharingFlowVisible(
                organizationAuthMetadata.getDisplayNameOfOrganization(OrganizationMetadataHelper.Organization.MIJN_AMSTERDAM),
                timeoutInSeconds = READER_STARTUP_TIMEOUT_SECONDS,
            ),
            "Disclosure screen not shown, reader output so far:\n$outputBuffer",
        )
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

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC80 Wallet does not contain requested attributes at close proximity disclosure")
    fun verifyCloseProximityWalletDoesNotContainRequestedAttributes(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        assertTrue(dashboardScreen.visible(), "Dashboard is not visible")
        dashboardScreen.showQRCode()
        closeProximityQrScreen.centerQr()
        val qrString = closeProximityQrScreen.getQr()
        val mockBleReaderApp = closeProximityQrScreen.startMockBleReaderApp(
            qrString,
            wrpacCaCrtFile = MONKEY_BIKE_CA_CRT_FILE,
            wrpacCaKeyFile = MONKEY_BIKE_CA_KEY_FILE,
            requestDocType = PID_DOC_TYPE,
            requestAttributes = MONKEY_BIKE_ATTRIBUTES,
        )
        val outputBuffer = mockBleReaderApp.captureOutput()
        assertTrue(
            attributesMissingErrorScreen.attributesMissingMessageVisible(timeoutInSeconds = READER_STARTUP_TIMEOUT_SECONDS),
            "Attributes missing message not visible, reader output so far:\n$outputBuffer",
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
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
            wrpacCaCrtFile = AMSTERDAM_CA_CRT_FILE,
            wrpacCaKeyFile = AMSTERDAM_CA_KEY_FILE,
            requestDocType = PID_DOC_TYPE,
            requestAttributes = AMSTERDAM_ATTRIBUTES,
            waitForDeviceResponse = true,
        )
        val outputBuffer = mockBleReaderApp.captureOutput()

        assertTrue(
            disclosureScreen.organizationNameForSharingFlowVisible(
                organizationAuthMetadata.getDisplayNameOfOrganization(OrganizationMetadataHelper.Organization.MIJN_AMSTERDAM),
                timeoutInSeconds = READER_STARTUP_TIMEOUT_SECONDS,
            ),
            "Disclosure screen not shown, reader output so far:\n$outputBuffer",
        )
        disclosureScreen.share()
        mockBleReaderApp.destroyForcibly()
        // On iOS the bluetooth disconnected screen is displayed almost immediately after the bluetooth disconnections drops
        // On Android the bluetooth disconnected screen is displayed after users enters their PIN
        if (disclosureScreen.platform() == Platform.ANDROID) {
            pinScreen.enterPin(DEFAULT_PIN)
        }
        assertTrue(bleDisconnectedScreen.visible(), "BLE disconnected screen is not visible")
    }
}
