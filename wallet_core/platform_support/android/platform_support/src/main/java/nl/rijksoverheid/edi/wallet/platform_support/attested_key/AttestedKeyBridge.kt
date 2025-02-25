package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.content.Context
import android.util.Log
import androidx.annotation.VisibleForTesting
import com.google.android.gms.tasks.Tasks
import com.google.android.play.core.integrity.IntegrityManagerFactory
import com.google.android.play.core.integrity.StandardIntegrityManager.PrepareIntegrityTokenRequest
import com.google.android.play.core.integrity.StandardIntegrityManager.StandardIntegrityToken
import com.google.android.play.core.integrity.StandardIntegrityManager.StandardIntegrityTokenProvider
import com.google.android.play.core.integrity.StandardIntegrityManager.StandardIntegrityTokenRequest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.keystore.KeyBridge
import nl.rijksoverheid.edi.wallet.platform_support.keystore.signing.SigningKey
import nl.rijksoverheid.edi.wallet.platform_support.util.toByteArray
import nl.rijksoverheid.edi.wallet.platform_support.util.toUByteList
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyException
import uniffi.platform_support.AttestedKeyType
import uniffi.platform_support.KeyStoreException
import java.util.UUID
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

    /**
     * Generate a new key pair with [identifier] and attestation [challenge].
     *
     * Note that this method should never throw [AttestedKeyException.ServerUnreachable], as that will cause the caller
     * to re-invoke this method with the same [identifier], which will fail. This behavior however is implemented specifically for
     * Apple app/key attestation.
     */
    @OptIn(ExperimentalUnsignedTypes::class, ExperimentalEncodingApi::class)
    @Throws(AttestedKeyException::class)
    override suspend fun attest(identifier: String, challenge: List<UByte>, googleCloudProjectNumber: ULong): AttestationData = withContext(Dispatchers.IO) {

        Log.d("attest", "beginning app and key attestation process")

        // TODO: PVW-4069: Handle non-existent or erronuous cloud project identifier numbers

        // Configure cloud project number, initialize manager and token provider request.
        Log.d("attest", "initializing using google cloud project number: $googleCloudProjectNumber")

        val integrityManager = IntegrityManagerFactory.createStandard(context.applicationContext)
        val integrityTokenProviderRequest =
            PrepareIntegrityTokenRequest.builder().setCloudProjectNumber(googleCloudProjectNumber.toLong()).build()

        // Execute integrity token provider request, await result, fill integrityTokenProvider.
        var integrityTokenProvider: StandardIntegrityTokenProvider = Tasks.await(
            integrityManager.prepareIntegrityToken(integrityTokenProviderRequest)
                .addOnSuccessListener { response ->
                    Log.d("attest", "configured integrity token provider")
                }
                .addOnFailureListener { exception ->
                    // TODO: PVW-4069: Consider throwing AttestedAppException.SomethingWithProvider equivalent here
                    Log.e("attest", exception.message ?: "there was no exception message")
                }
        )

        // Execute integrity token request using provider.
        val integrityTokenRequest = StandardIntegrityTokenRequest.builder().setRequestHash(Base64.Default.encode(challenge.toByteArray())).build()
        val integrityToken: StandardIntegrityToken = Tasks.await (integrityTokenProvider.request(integrityTokenRequest)
            .addOnSuccessListener { response ->
                Log.i("attest", "received an integrity token")
            }
            .addOnFailureListener { exception ->
                // TODO: PVW-4069: Consider throwing AttestedAppException.SomethingWithToken equivalent here
                Log.e("attest", exception.message ?: "there was no exception message")
            }
        )

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
            when (e) {
                is AttestedKeyException -> throw e
                is java.security.KeyStoreException -> throw AttestedKeyException.Other("failed to get certificate chain: ${e.message}")
                else -> throw AttestedKeyException.Other("unexpected failure: ${e.message}")
            }
        }
    }

    @Throws(AttestedKeyException::class)
    override suspend fun sign(identifier: String, payload: List<UByte>): List<UByte> = withContext(Dispatchers.IO) {
        try {
            getKey(identifier.keyAlias()).sign(payload)
        } catch (e: Exception) {
            when (e) {
                is AttestedKeyException -> throw e
                is KeyStoreException.KeyException -> throw AttestedKeyException.Other("failed to sign the payload: ${e.message}")
                else -> throw AttestedKeyException.Other("unexpected failure: ${e.message}")
            }
        }
    }

    override suspend fun publicKey(identifier: String): List<UByte> = withContext(Dispatchers.IO) {
        try {
            getKey(identifier.keyAlias()).publicKey()
        } catch (e: Exception) {
            when (e) {
                is AttestedKeyException -> throw e
                is KeyStoreException.KeyException -> throw AttestedKeyException.Other("failed to obtain public key: ${e.message}")
                else -> throw AttestedKeyException.Other("unexpected failure: ${e.message}")
            }
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
        } catch (e: java.security.KeyStoreException) {
            throw AttestedKeyException.Other("failed to delete keystore entry: ${e.message}")
        }
    }

    @Throws(AttestedKeyException::class)
    private fun createKey(identifier: String, challenge: List<UByte>): SigningKey {
        val keyAlias = identifier.keyAlias()
        try {
            verifyDeviceUnlocked()
            verifyKeyDoesNotExist(keyAlias)
            SigningKey.createKey(context, keyAlias, challenge)
            return SigningKey(keyAlias).takeIf { it.isConsideredValid }!!
        } catch (e: Exception) {
            throw when (e) {
                is IllegalStateException -> throw AttestedKeyException.Other("precondition failed: ${e.message}")
                is NullPointerException -> throw AttestedKeyException.Other("generated key is invalid")
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
