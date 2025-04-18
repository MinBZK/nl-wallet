package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.content.Context
import android.util.Log
import androidx.annotation.VisibleForTesting
import com.google.android.play.core.integrity.IntegrityManagerFactory
import com.google.android.play.core.integrity.StandardIntegrityManager.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.tasks.await
import kotlinx.coroutines.withContext
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SigningKey
import nl.rijksoverheid.edi.wallet.platform_support.utilities.retryable
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.utilities.toUByteList
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyException
import uniffi.platform_support.AttestedKeyType
import uniffi.platform_support.KeyStoreException
import java.util.*
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import uniffi.platform_support.AttestedKeyBridge as RustAttestedKeyBridge

// Note this prefix is almost the same as [SigningKeyBridge.SIGN_KEY_PREFIX], however this one ends with a hyphen '-'.
private const val ATTESTED_KEY_PREFIX = "ecdsa_"

private fun String.keyAlias() = ATTESTED_KEY_PREFIX + this

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class AttestedKeyBridge(context: Context) : KeyBridge(context), RustAttestedKeyBridge {

    override fun keyType(): AttestedKeyType = AttestedKeyType.GOOGLE

    override suspend fun generate(): String = UUID.randomUUID().toString()

    private val mutex = Mutex() // Mutex guards next variables
    private lateinit var integrityTokenProvider : StandardIntegrityTokenProvider
    private var currentGoogleCloudProjectNumber: ULong = 0u

    private suspend fun initializeIntegrityTokenProvider(googleCloudProjectNumber: ULong) {
        mutex.withLock {
            // Immediately return the integrity token provider if initialized and cloud project number is unchanged.
            if (::integrityTokenProvider.isInitialized && currentGoogleCloudProjectNumber == googleCloudProjectNumber) {
                return@withLock
            }

            // Configure cloud project number, initialize manager and token provider request.
            Log.d("attest", "initializing integrity token provider using google cloud project number: $googleCloudProjectNumber")

            // Initialize integrity manager.
            val integrityManager = IntegrityManagerFactory.createStandard(context.applicationContext)

            // Initialize integrity token provider request.
            val integrityTokenProviderRequest = PrepareIntegrityTokenRequest.builder()
                .setCloudProjectNumber(googleCloudProjectNumber.toLong())
                .build()

            // Retryably execute integrity token provider request, await result, fill integrityTokenProvider.
            integrityTokenProvider = retryable(
                taskName = "attest",
                taskDescription = "obtaining integrity token provider"
            ) { integrityManager.prepareIntegrityToken(integrityTokenProviderRequest).await() }

            // Set last as this is used to check for init.
            currentGoogleCloudProjectNumber = googleCloudProjectNumber
        }
    }

    /**
     * Generate a new key pair with [identifier] and attestation [challenge].
     *
     * Note that this method should never throw [AttestedKeyException.ServerUnreachable], as that will cause the caller
     * to re-invoke this method with the same [identifier], which will fail. This behavior however is implemented specifically for
     * Apple app/key attestation.
     */
    @OptIn(ExperimentalEncodingApi::class)
    @Throws(AttestedKeyException::class)
    override suspend fun attest(identifier: String, challenge: List<UByte>, googleCloudProjectNumber: ULong): AttestationData = withContext(Dispatchers.IO) {
        // Initialize integrity token provider.
        initializeIntegrityTokenProvider(googleCloudProjectNumber)

        // Retryably execute integrity token request using provider, await result, fill integrityToken.
        val integrityTokenRequest = StandardIntegrityTokenRequest.builder().setRequestHash(Base64.Default.encode(challenge.toByteArray())).build()
        val integrityToken = retryable(
            taskName = "attest",
            taskDescription = "obtaining integrity token"
        ) { integrityTokenProvider.request(integrityTokenRequest).await() }

        val signingKey = createKey(identifier, challenge)
        try {
            val certificateChain =
                signingKey.getCertificateChain()?.asSequence()?.map { it.encoded.toUByteList() }?.toList()
                    ?: throw AttestedKeyException.Other("failed to get certificate chain")

            val appAttestationToken = integrityToken.token()

            AttestationData.Google(certificateChain, appAttestationToken)
        } catch (e: Exception) {
            // Try to undo the creation of the signing key
            try {
                delete(identifier)
            } catch (ex: Exception) {
                Log.w("attest", "failed to delete key with id '$identifier'", ex)
            }
            e.wrapAndThrow("failed to attest key")
        }
    }

    @Throws(AttestedKeyException::class)
    override suspend fun sign(identifier: String, payload: List<UByte>): List<UByte> = withContext(Dispatchers.IO) {
        try {
            getKey(identifier.keyAlias()).sign(payload)
        } catch (e: Exception) {
            e.wrapAndThrow("failed to sign the payload")
        }
    }

    override suspend fun publicKey(identifier: String): List<UByte> = withContext(Dispatchers.IO) {
        try {
            getKey(identifier.keyAlias()).publicKey()
        } catch (e: Exception) {
            e.wrapAndThrow("failed to obtain public key")
        }
    }

    /**
     * Delete key with [identifier] if it exists.
     * If no key with [identifier] exists, this method will not throw an exception.
     * @throws AttestedKeyException if the entry cannot be removed.
     */
    override suspend fun delete(identifier: String) = withContext(Dispatchers.IO) {
        try {
            keyStore.deleteEntry(identifier.keyAlias())
        } catch (e: Exception) {
            e.wrapAndThrow("failed to delete key")
        }
    }

    private fun Exception.wrapAndThrow(keyExceptionReason: String): Nothing =
        when (this) {
            is AttestedKeyException -> throw this
            is KeyStoreException.KeyException -> throw AttestedKeyException.Other("$keyExceptionReason: $message")
            else -> throw AttestedKeyException.Other("unexpected failure: $message")
        }

    @Throws(AttestedKeyException::class)
    private fun createKey(identifier: String, challenge: List<UByte>): SigningKey {
        val keyAlias = identifier.keyAlias()
        try {
            verifyDeviceUnlocked()
            verifyKeyDoesNotExist(keyAlias)
            SigningKey.createKey(context, keyAlias, challenge)
            return SigningKey(keyAlias)
        } catch (e: Exception) {
            throw when (e) {
                is IllegalStateException -> throw AttestedKeyException.Other("precondition failed: ${e.message}")
                else -> AttestedKeyException.Other("failed to create key: ${e.message}")
            }
        }
    }

    @Throws(AttestedKeyException::class)
    private fun getKey(keyAlias: String): SigningKey {
        try {
            verifyDeviceUnlocked()
            verifyKeyExists(keyAlias)
        } catch (e: IllegalStateException) {
            throw AttestedKeyException.Other("precondition failed: ${e.message}")
        }
        return SigningKey(keyAlias)
    }

    @VisibleForTesting
    override fun clean() =
        keyStore.aliases().asSequence().filter { it.startsWith(ATTESTED_KEY_PREFIX) }.forEach(::deleteEntry)
}
