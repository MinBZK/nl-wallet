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
        private val AMSTERDAM_WRPAC_CRT_FILE = System.getenv("AMSTERDAM_WRPAC_CRT_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/mijn_amsterdam.crt.pem").canonicalPath
        private val AMSTERDAM_WRPAC_KEY_FILE = System.getenv("AMSTERDAM_WRPAC_KEY_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/mijn_amsterdam.key.pem").canonicalPath
        private val AMSTERDAM_WRPRC_FILE = System.getenv("AMSTERDAM_WRPRC_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/mijn_amsterdam.wrprc").canonicalPath
        private val MONKEY_BIKE_WRPAC_CRT_FILE = System.getenv("MONKEY_BIKE_WRPAC_CRT_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/monkey_bike.crt.pem").canonicalPath
        private val MONKEY_BIKE_WRPAC_KEY_FILE = System.getenv("MONKEY_BIKE_WRPAC_KEY_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/monkey_bike.key.pem").canonicalPath
        private val MONKEY_BIKE_WRPRC_FILE = System.getenv("MONKEY_BIKE_WRPRC_FILE")
            ?: File("../scripts/devenv/target/demo_relying_party/monkey_bike.wrprc").canonicalPath

        private const val PID_DOC_TYPE = "eu.europa.ec.eudi.pid.1"
        private const val PID_NAMESPACE = "eu.europa.ec.eudi.pid.1"
        private const val PID_NL_NAMESPACE = "eu.europa.ec.eudi.pid.nl.1"
        private val AMSTERDAM_ATTRIBUTES = listOf("$PID_NL_NAMESPACE/bsn")
        // `gender` is not part of the PID, which makes this request fail with missing attributes
        private val MONKEY_BIKE_ATTRIBUTES = listOf(
            "$PID_NAMESPACE/given_name",
            "$PID_NAMESPACE/family_name",
            "$PID_NAMESPACE/birth_date",
            "$PID_NAMESPACE/gender",
            "$PID_NAMESPACE/resident_street",
            "$PID_NAMESPACE/resident_postal_code",
            "$PID_NAMESPACE/resident_city",
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
        dashboardScreen.captureA11ySnapshot("Dashboard")
        dashboardScreen.showQRCode()
        closeProximityQrScreen.centerQr()
        closeProximityQrScreen.captureA11ySnapshot("CloseProximityQr")
        val qrString = closeProximityQrScreen.getQr()
        val mockBleReaderApp = closeProximityQrScreen.startMockBleReaderApp(
            qrString,
            wrpacCrtFile = AMSTERDAM_WRPAC_CRT_FILE,
            wrpacKeyFile = AMSTERDAM_WRPAC_KEY_FILE,
            wrprcFile = AMSTERDAM_WRPRC_FILE,
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
        disclosureScreen.captureA11ySnapshot("DisclosureApproveOrganization")
        disclosureScreen.share()
        pinScreen.enterPin(DEFAULT_PIN)
        disclosureScreen.goToDashBoard()
        val exitCode = mockBleReaderApp.waitFor()
        val hex = DeviceResponseHelper.extractHex(outputBuffer.toString())
        val deviceResponse = DeviceResponseHelper.parse(hex!!)
        val doc = deviceResponse.documents.first()
        val bsn = doc.attributes.firstOrNull { it.namespace == PID_NL_NAMESPACE && it.identifier == "bsn" }?.value

        assertAll(
            { assertTrue(dashboardScreen.visible(), "Dashboard is not visible") },
            { assertTrue(exitCode == 0, "Mac reader failed (exit $exitCode):\n$outputBuffer" ) },
            { assertTrue(deviceResponse.version == "1.0", "Device response version mismatch") },
            { assertTrue(deviceResponse.status == 0, "Device response status is not success") },
            { assertTrue(deviceResponse.documents.size == 1, "Expected exactly one document") },
            { assertTrue(doc.docType == PID_DOC_TYPE, "Document type mismatch") },
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
            wrpacCrtFile = MONKEY_BIKE_WRPAC_CRT_FILE,
            wrpacKeyFile = MONKEY_BIKE_WRPAC_KEY_FILE,
            wrprcFile = MONKEY_BIKE_WRPRC_FILE,
            requestDocType = PID_DOC_TYPE,
            requestAttributes = MONKEY_BIKE_ATTRIBUTES,
        )
        val outputBuffer = mockBleReaderApp.captureOutput()
        assertTrue(
            attributesMissingErrorScreen.attributesMissingMessageVisible(timeoutInSeconds = READER_STARTUP_TIMEOUT_SECONDS),
            "Attributes missing message not visible, reader output so far:\n$outputBuffer",
        )
        attributesMissingErrorScreen.captureA11ySnapshot("AttributesMissingError")
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
            wrpacCrtFile = AMSTERDAM_WRPAC_CRT_FILE,
            wrpacKeyFile = AMSTERDAM_WRPAC_KEY_FILE,
            wrprcFile = AMSTERDAM_WRPRC_FILE,
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
        bleDisconnectedScreen.captureA11ySnapshot("BleDisconnected")
    }
}
