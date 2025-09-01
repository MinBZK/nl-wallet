package nl.rijksoverheid.edi.wallet.platform_support.keystore

import org.hamcrest.CoreMatchers.isA
import org.hamcrest.MatcherAssert.assertThat
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.platform_support.KeyStoreException

class KeyExceptionBuilderTest {
    @Test
    fun `deriveKeyError returns KeyException and contains the provided reason`() {
        val reason = "test reason"
        val ex = Exception(reason)
        val keyException = KeyExceptionBuilder.deriveKeyError(ex)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains(reason))
    }

    @Test
    fun `signKeyError returns KeyException and contains the provided reason`() {
        val reason = "sign fail"
        val ex = Exception(reason)
        val keyException = KeyExceptionBuilder.signKeyError(ex)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains(reason))
    }

    @Test
    fun `createKeyError returns KeyException and contains the provided reason`() {
        val reason = "create fail"
        val ex = Exception(reason)
        val keyException = KeyExceptionBuilder.createKeyError(ex)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains(reason))
    }

    @Test
    fun `fetchKeyError returns KeyException and contains the provided reason`() {
        val reason = "fetch fail"
        val ex = Exception(reason)
        val keyException = KeyExceptionBuilder.fetchKeyError(ex)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains(reason))
    }

    @Test
    fun `certificateChainError returns KeyException and contains the provided reason`() {
        val reason = "chain fail"
        val ex = Exception(reason)
        val keyException = KeyExceptionBuilder.certificateChainError(ex)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains(reason))
    }

    @Test
    fun `missingHardwareError returns KeyException and contains security level`() {
        val keySecurityLevel = 42
        val keyException = KeyExceptionBuilder.missingHardwareError(keySecurityLevel)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains(keySecurityLevel.toString()))
    }

    @Test
    fun `missingHardwareError returns KeyException and handles null security level`() {
        val keyException = KeyExceptionBuilder.missingHardwareError(null)
        assertThat(keyException, isA(KeyStoreException.KeyException::class.java))
        assertTrue(keyException.message.contains("null"))
    }
}