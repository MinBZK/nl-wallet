package nl.rijksoverheid.edi.wallet.integration_tests

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.RequiresDevice
import org.junit.Assert.assertNull
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
@RequiresDevice
class CloseProximityDisclosureTests : WalletDeviceTest() {
    @Test
    fun testWalletCloseProximityDisclosureWithMacReader() {
        val error = WalletTestLibrary.runDisclosure()
        assertNull(error, error)
    }

    @Test
    fun testWalletJwtRegistrationCertificateWithMacReader() {
        val error = WalletTestLibrary.runDisclosure(jwtRegistrationCertificate = true)
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateMissing() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "missing")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateWrongType() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "wrong_type")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateMalformed() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "malformed")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateTamperedSignature() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "tampered_signature")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateExpired() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "expired")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateUntrusted() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "untrusted")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateMismatchedReader() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "mismatched_reader")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateMismatchedSubjectType() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "mismatched_subject_type")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateRevoked() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "revoked")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateSuspended() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "suspended")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateStatusUnavailable() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "status_unavailable")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateUntrustedStatusList() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "untrusted_status_list")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateStatusIndexOutOfBounds() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "status_index_out_of_bounds")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateUnauthorizedDocumentType() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "unauthorized_document_type")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateUnauthorizedNamespace() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "unauthorized_namespace")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateUnauthorizedAttribute() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "unauthorized_attribute")
        assertNull(error, error)
    }

    @Test
    fun testWalletRegistrationCertificateUnauthorizedSecondDocument() {
        val error = WalletTestLibrary.runDisclosure(registrationCase = "unauthorized_second_document")
        assertNull(error, error)
    }
}
