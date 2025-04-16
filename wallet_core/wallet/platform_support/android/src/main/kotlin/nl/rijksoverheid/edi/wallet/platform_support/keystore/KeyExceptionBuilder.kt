package nl.rijksoverheid.edi.wallet.platform_support.keystore

import uniffi.platform_support.KeyStoreException.KeyException

/**
 * Wrapper for the errors that can occur when manipulating keys in
 * the keystore. Counterpart to iOS's [SecureEnclaveKeyError.swift].
 */
object KeyExceptionBuilder {
    fun deriveKeyError(ex: Exception) = ex.toKeyException("Could not derive public key")
    fun signKeyError(ex: Exception) = ex.toKeyException("Could not sign with private key")
    fun createKeyError(ex: Exception) = ex.toKeyException("Could not create private key")
    fun fetchKeyError(ex: Exception) = ex.toKeyException("Could not fetch private key")
    fun certificateChainError(ex: Exception) = ex.toKeyException("Could not fetch certificate chain of key")
    fun missingHardwareError(keySecurityLevel: Int?)
        = KeyException("Could not generate hardware backed key. Key security level: $keySecurityLevel")

    private fun Exception.toKeyException(errorMessage: String) = KeyException("$errorMessage. Reason: $message")
}
