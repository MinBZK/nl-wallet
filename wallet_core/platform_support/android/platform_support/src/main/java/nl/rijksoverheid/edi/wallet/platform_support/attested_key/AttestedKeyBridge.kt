package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.utilities.storage.StoragePathProvider
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyBridge as RustAttestedKeyBridge
import uniffi.platform_support.AttestedKeyException.MethodUnimplemented
import uniffi.platform_support.AttestedKeyType

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class AttestedKeyBridge(context: Context) : RustAttestedKeyBridge {

    override fun keyType(): AttestedKeyType = AttestedKeyType.GOOGLE

    override suspend fun generate(): String {
        throw MethodUnimplemented()
    }
    
    override suspend fun attest(identifier: String, challenge: List<UByte>): AttestationData {
        throw MethodUnimplemented()
    }
    
    override suspend fun sign(identifier: String, payload: List<UByte>): List<UByte> {
        throw MethodUnimplemented()
    }
    
    override suspend fun publicKey(identifier: String): List<UByte> {
        throw MethodUnimplemented()
    }
    
    override suspend fun delete(identifier: String) {
        throw MethodUnimplemented()
    }
}
