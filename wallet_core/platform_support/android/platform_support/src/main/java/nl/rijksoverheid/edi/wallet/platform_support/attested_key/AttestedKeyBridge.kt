package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.content.Context
import android.os.Build
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyStoreKeyError
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SigningKey
import nl.rijksoverheid.edi.wallet.platform_support.longRunning
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyException
import uniffi.platform_support.AttestedKeyType
import uniffi.platform_support.KeyStoreException
import java.util.UUID
import uniffi.platform_support.AttestedKeyBridge as RustAttestedKeyBridge

// Note this prefix is almost the same as [SigningKeyBridge.SIGN_KEY_PREFIX], however this one ends with a hyphen '-'.
private const val ATTESTED_KEY_PREFIX = "ecdsa-"

private fun String.keyAlias() = ATTESTED_KEY_PREFIX + this

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class AttestedKeyBridge(context: Context) : KeyBridge(context), RustAttestedKeyBridge {

    override fun keyType(): AttestedKeyType = AttestedKeyType.GOOGLE

    override suspend fun generate(): String = UUID.randomUUID().toString()

    @Throws(AttestedKeyException::class)
    override suspend fun attest(identifier: String, challenge: List<UByte>): AttestationData = longRunning {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
            // TODO: Is this correct? Only the [setAttestationChallenge] method was added then, is it strictly necessary?
            throw AttestedKeyException.AttestationNotSupported()
        }

        val signingKey = createKey(identifier, challenge)
        val challengeResponse = signingKey.sign(challenge)
        val certificateChain = signingKey.getCertificateChain()
            ?.asSequence()
            ?.map { it.encoded.toUByteList() }
            ?.toList() ?: throw AttestedKeyException.Other("failed to get Certificate Chain")

        AttestationData.Google(
            certificateChain,
            challengeResponse
        )
    }

    @Throws(AttestedKeyException::class)
    override suspend fun sign(identifier: String, payload: List<UByte>): List<UByte> = longRunning {
        try {
            getKey(identifier.keyAlias()).sign(payload)
        } catch (e: Exception) {
            when (e) {
                is IllegalStateException -> throw AttestedKeyException.Other("precondition failed: ${e.message}")
                is KeyStoreException.KeyException -> throw AttestedKeyException.Other("failed to obtain public key: ${e.message}")
                else -> throw AttestedKeyException.Other("unexpected failure: ${e.message}")
            }
        }
    }


    override suspend fun publicKey(identifier: String): List<UByte> = longRunning {
        try {
            getKey(identifier.keyAlias()).publicKey()
        } catch (e: Exception) {
            when (e) {
                is IllegalStateException -> throw AttestedKeyException.Other("precondition failed: ${e.message}")
                is KeyStoreException.KeyException -> throw AttestedKeyException.Other("failed to obtain public key: ${e.message}")
                else -> throw AttestedKeyException.Other("unexpected failure: ${e.message}")
            }
        }
    }

    override suspend fun delete(identifier: String) = longRunning {
        try {
            keyStore.deleteEntry(identifier.keyAlias())
        } catch (e: java.security.KeyStoreException) {
            throw AttestedKeyException.Other("failed to delete keystore entry: ${e.message}")
        }
    }

    @Throws(KeyStoreException::class, IllegalStateException::class)
    private fun createKey(identifier: String, challenge: List<UByte>): SigningKey {
        val keyAlias = identifier.keyAlias()
        try {
            verifyDeviceUnlocked()
            verifyKeyDoesNotExist(keyAlias)
            SigningKey.createKey(context, keyAlias, challenge)
            return SigningKey(keyAlias).takeIf { it.isConsideredValid }!!
        } catch (ex: Exception) {
            if (ex is KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    @Throws(IllegalStateException::class)
    private fun getKey(keyAlias: String): SigningKey {
        verifyDeviceUnlocked()
        verifyKeyExists(keyAlias)
        return SigningKey(keyAlias)
    }

    override fun clean() =
        keyStore.aliases().asSequence().filter { it.startsWith(ATTESTED_KEY_PREFIX) }
            .forEach(::deleteEntry)
}
