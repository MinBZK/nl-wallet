package feature.disclosure

import helper.OrganizationAuthMetadataHelper
import helper.OrganizationAuthMetadataHelper.Organization.INSURANCE
import helper.TestBase
import navigator.MenuNavigator
import navigator.OnboardingNavigator
import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import org.json.JSONObject
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junit.jupiter.api.assertAll
import org.junitpioneer.jupiter.RetryingTest
import screen.dashboard.DashboardScreen
import screen.demo.DemoScreen
import screen.disclosure.ScanWithWalletDialog
import screen.error.InvalidIssuanceULErrorScreen
import screen.issuance.DisclosureIssuanceScreen
import screen.issuance.FinishWalletDialog
import screen.security.PinScreen
import java.net.URI
import java.net.URLDecoder
import java.net.URLEncoder
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.security.MessageDigest
import java.util.Base64

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("Partial Flow 2.7 Resolve a universal link")
class UniversalLinkTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen
    private lateinit var scanWithWalletDialog: ScanWithWalletDialog
    private lateinit var expiredDisclosureUniversalLinkFromCameraApp: String
    private lateinit var demoScreen: DemoScreen
    private lateinit var invalidIssuanceULErrorScreen: InvalidIssuanceULErrorScreen
    private lateinit var disclosureForIssuanceScreen: DisclosureIssuanceScreen
    private lateinit var organizationAuthMetadata: OrganizationAuthMetadataHelper
    private lateinit var pinScreen: PinScreen
    private lateinit var finishWalletDialog: FinishWalletDialog

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        expiredDisclosureUniversalLinkFromCameraApp = buildUniversalLink(
            DISCLOSURE_UNIVERSAL_LINK_BASE,
            mapOf(
                "request_uri" to EXPIRED_DISCLOSURE_REQUEST_URI,
                "request_uri_method" to POST_REQUEST_URI_METHOD,
                "client_id" to resolveUsecaseX509HashClientId(
                    MIJN_AMSTERDAM_START_URL,
                    "mijn_amsterdam_mdoc",
                    CROSS_DEVICE_SESSION_TYPE,
                ),
            )
        )
        dashboardScreen = DashboardScreen()
        demoScreen = DemoScreen()
        invalidIssuanceULErrorScreen = InvalidIssuanceULErrorScreen()
        scanWithWalletDialog = ScanWithWalletDialog()
        organizationAuthMetadata = OrganizationAuthMetadataHelper()
        disclosureForIssuanceScreen = DisclosureIssuanceScreen()
        pinScreen = PinScreen()
        finishWalletDialog = FinishWalletDialog()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC33 Open app via universal link")
    fun verifyUlOpensApp(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        val issuanceUniversalLink = buildIssuanceUniversalLink()
        dashboardScreen.closeApp()
        dashboardScreen.openLink(issuanceUniversalLink)
        pinScreen.enterPin(DEFAULT_PIN)

        assertTrue(
            disclosureForIssuanceScreen.organizationNameVisible(
                organizationAuthMetadata.getAttributeValueForOrganization("organization.displayName", INSURANCE)
            )
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC36 Universal link is opened via external QR scanner")
    fun verifyScanInAppDialog(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Menu)
        dashboardScreen.openLink(expiredDisclosureUniversalLinkFromCameraApp)
        assertAll(
            { assertTrue(scanWithWalletDialog.visible(), "scan with wallet dialog is not visible") },
            { assertTrue(scanWithWalletDialog.scanWithWalletDialogBodyVisible(), "scan with wallet dialog subtitle is not visible") },
            { assertTrue(scanWithWalletDialog.scanWithWalletButtonVisible(), "scan with wallet button is not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC44 Wallet not created when universal link is invoked")
    fun verifyWhenAppNotActivated(testInfo: TestInfo) {
        setUp(testInfo)
        demoScreen.openLink(expiredDisclosureUniversalLinkFromCameraApp)
        finishWalletDialog.clickOkButton()
        assertTrue(demoScreen.visible(), "demo screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC6 Invalid universal link results in error screen")
    fun verifyInvalidUniversalLink(testInfo: TestInfo) {
        setUp(testInfo)
        MenuNavigator().toScreen(MenuNavigatorScreen.Dashboard)
        val invalidIssuanceUniversalLink = buildInvalidIssuanceUniversalLink()
        dashboardScreen.openLink(invalidIssuanceUniversalLink)
        assertAll(
            { assertTrue(invalidIssuanceULErrorScreen.headlineVisible(), "Headline is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.closeButtonVisible(), "Close button is not visible") },
        )
        invalidIssuanceULErrorScreen.errorDetails.seeDetails()
        assertAll(
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appVersionLabelVisible(), "App version label is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.osVersionLabelVisible(), "OS version label is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appConfigLabelVisible(), "App config label is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appVersionVisible(), "App version is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.osVersionVisible(), "OS version is not visible") },
            { assertTrue(invalidIssuanceULErrorScreen.errorDetails.appConfigVisible(), "App config is not visible") },
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("LTC69 universal link is invoked while wallet is being personalized")
    fun verifyWhenAppIsBeingPersonalized(testInfo: TestInfo) {
        setUp(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.SecurityChoosePin)
        pinScreen.openLink(expiredDisclosureUniversalLinkFromCameraApp)
        assertTrue(finishWalletDialog.visible(), "Finish wallet dialog is not visible")
    }

    private fun buildIssuanceUniversalLink(): String {
        val authorizationRequestDetails = fetchAuthorizationRequestDetails(
            INSURANCE_ISSUANCE_REQUEST_URI,
            POST_REQUEST_URI_METHOD,
        )

        return buildUniversalLink(
            DISCLOSURE_BASED_ISSUANCE_UNIVERSAL_LINK_BASE,
            mapOf(
                "client_id" to authorizationRequestDetails.universalLinkClientId(),
                "request_uri" to INSURANCE_ISSUANCE_REQUEST_URI,
                "request_uri_method" to POST_REQUEST_URI_METHOD,
            )
        )
    }

    private fun buildInvalidIssuanceUniversalLink(): String {
        val authorizationRequestDetails = fetchAuthorizationRequestDetails(
            INSURANCE_ISSUANCE_REQUEST_URI,
            POST_REQUEST_URI_METHOD,
        )

        return buildUniversalLink(
            DISCLOSURE_BASED_ISSUANCE_UNIVERSAL_LINK_BASE,
            mapOf(
                "request_uri" to INSURANCE_ISSUANCE_REQUEST_URI,
                "request_uri_method" to POST_REQUEST_URI_METHOD,
                "client_id" to invalidateX509HashClientId(authorizationRequestDetails.x509HashClientId),
            )
        )
    }

    private fun resolveUsecaseX509HashClientId(startUrl: String, usecase: String, sessionType: String): String {
        val sessionResponse = JSONObject(
            sendRequest(
                HttpRequest.newBuilder(URI(startUrl))
                    .header("Content-Type", "application/json")
                    .POST(HttpRequest.BodyPublishers.ofString(JSONObject(mapOf("usecase" to usecase)).toString()))
                    .build()
            )
        )
        val statusUrl = sessionResponse.getString("status_url")
        val statusResponse = JSONObject(sendRequest(HttpRequest.newBuilder(URI("$statusUrl?session_type=$sessionType")).GET().build()))
        val requestParameters = queryParameters(statusResponse.getString("ul"))
        val requestUri = requestParameters.getValue("request_uri")
        val requestUriMethod = requestParameters.getValue("request_uri_method")

        return fetchAuthorizationRequestDetails(requestUri, requestUriMethod).x509HashClientId
    }

    private fun fetchAuthorizationRequestDetails(requestUri: String, requestUriMethod: String): AuthorizationRequestDetails {
        val authorizationRequestJwt = fetchAuthorizationRequest(requestUri, requestUriMethod)
        val jwtSegments = authorizationRequestJwt.split('.')

        require(jwtSegments.size == 3) { "Expected Authorization Request JWT, got: $authorizationRequestJwt" }

        val header = JSONObject(decodeJwtSegment(jwtSegments[0]))
        val payload = JSONObject(decodeJwtSegment(jwtSegments[1]))
        val x509HashClientId = "x509_hash:${computeLeafCertificateHash(header)}"

        return AuthorizationRequestDetails(
            clientId = payload.getString("client_id"),
            x509HashClientId = x509HashClientId,
        )
    }

    private fun fetchAuthorizationRequest(requestUri: String, requestUriMethod: String): String {
        val requestBuilder = HttpRequest.newBuilder(URI(requestUri))
        val request = when (requestUriMethod.lowercase()) {
            GET_REQUEST_URI_METHOD -> requestBuilder.GET().build()
            POST_REQUEST_URI_METHOD -> requestBuilder
                .header("Content-Type", "application/x-www-form-urlencoded")
                .POST(HttpRequest.BodyPublishers.noBody())
                .build()

            else -> error("Unsupported request_uri_method: $requestUriMethod")
        }

        return sendRequest(request)
    }

    private fun sendRequest(request: HttpRequest): String {
        val response = httpClient.send(request, HttpResponse.BodyHandlers.ofString())
        check(response.statusCode() == 200) {
            "Unexpected HTTP ${response.statusCode()} for ${request.method()} ${request.uri()}"
        }

        return response.body().trim()
    }

    private fun computeLeafCertificateHash(header: JSONObject): String {
        val certificates = header.getJSONArray("x5c")
        check(certificates.length() > 0) { "Authorization Request x5c header is empty" }

        val leafCertificate = Base64.getDecoder().decode(certificates.getString(0))
        val certificateHash = MessageDigest.getInstance("SHA-256").digest(leafCertificate)

        return Base64.getUrlEncoder().withoutPadding().encodeToString(certificateHash)
    }

    private fun decodeJwtSegment(segment: String): String {
        val padding = (4 - segment.length % 4) % 4
        val decoded = Base64.getUrlDecoder().decode(segment + "=".repeat(padding))
        return String(decoded, Charsets.UTF_8)
    }

    private fun queryParameters(url: String): Map<String, String> {
        val rawQuery = requireNotNull(URI(url).rawQuery) { "Expected query string in $url" }

        return rawQuery.split("&").associate { entry ->
            val (key, value) = entry.split("=", limit = 2).let { parts ->
                parts[0] to parts.getOrElse(1) { "" }
            }

            URLDecoder.decode(key, Charsets.UTF_8) to URLDecoder.decode(value, Charsets.UTF_8)
        }
    }

    private fun buildUniversalLink(base: String, parameters: Map<String, String>): String =
        "$base?" + parameters.entries.joinToString("&") {
            "${it.key}=${URLEncoder.encode(it.value, Charsets.UTF_8)}"
        }

    private fun invalidateX509HashClientId(x509HashClientId: String): String {
        require(x509HashClientId.startsWith("x509_hash:")) {
            "Expected x509_hash client_id, got $x509HashClientId"
        }

        val hash = x509HashClientId.removePrefix("x509_hash:")
        val replacement = if (hash.last() == 'A') 'B' else 'A'

        return "x509_hash:${hash.dropLast(1)}$replacement"
    }

    private fun AuthorizationRequestDetails.universalLinkClientId(): String {
        if (!clientId.startsWith("x509_hash:")) {
            return clientId
        }

        require(clientId == x509HashClientId) {
            "Authorization Request client_id $clientId does not match x5c leaf hash $x509HashClientId"
        }

        return x509HashClientId
    }

    private data class AuthorizationRequestDetails(
        val clientId: String,
        val x509HashClientId: String,
    )

    companion object {
        private val httpClient = HttpClient.newHttpClient()

        private const val DISCLOSURE_UNIVERSAL_LINK_BASE = "https://app.example.com/deeplink/disclosure"
        private const val DISCLOSURE_BASED_ISSUANCE_UNIVERSAL_LINK_BASE =
            "https://app.example.com/deeplink/disclosure_based_issuance"
        private const val EXPIRED_DISCLOSURE_REQUEST_URI =
            "https://example.com/disclosure/sessions/CYqJdDLRIkFArxoWLXLUYaAkUiK4A6YF/request_uri?session_type=cross_device&ephemeral_id=02a1bf4d24a54228be1ba88576bfd4d7df8759d23df90822fda8f49da6826213&time=2025-04-10T10%3A44%3A15.629765875Z"
        private const val INSURANCE_ISSUANCE_REQUEST_URI =
            "https://example.com/cd96997cf3772b54a9a0c9f2d261a401/disclosure/insurance/request_uri?session_type=same_device"
        private const val MIJN_AMSTERDAM_START_URL =
            "https://example.com/sessions?lang=en"
        private const val CROSS_DEVICE_SESSION_TYPE = "cross_device"
        private const val GET_REQUEST_URI_METHOD = "get"
        private const val POST_REQUEST_URI_METHOD = "post"
    }
}
