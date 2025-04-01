package nl.rijksoverheid.edi.wallet.platform_support.keystore

import uniffi.platform_support.KeyStoreException.*

/**
 * Wrapper for the errors that can occur when manipulating keys in
 * the keystore. Counterpart to iOS's [SecureEnclaveKeyError.swift].
 */
sealed class KeyStoreKeyError(private val ex: Exception) {
    class DeriveKeyError(ex: Exception) : KeyStoreKeyError(ex)
    class SignKeyError(ex: Exception) : KeyStoreKeyError(ex)
    class CreateKeyError(ex: Exception) : KeyStoreKeyError(ex)
    class FetchKeyError(ex: Exception) : KeyStoreKeyError(ex)
    class MissingHardwareError(keySecurityLevel: Int?) :
        KeyStoreKeyError(Exception("Key security level: $keySecurityLevel"))

    val keyException: KeyException
        get() {
            val errorMessage = when (this) {
                is DeriveKeyError -> "Could not derive public key"
                is SignKeyError -> "Could not sign with private key"
                is CreateKeyError -> "Could not create private key"
                is FetchKeyError -> "Could not fetch private key"
                is MissingHardwareError -> "Could not generate hardware backed key"
            }
            return KeyException("$errorMessage. Reason: ${ex.message}")
        }
}
