package nl.rijksoverheid.edi.wallet.platform_support.attested_key

import android.content.Context
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.utilities.storage.StoragePathProvider
import uniffi.platform_support.AttestationData
import uniffi.platform_support.AttestedKeyBridge as RustAttestedKeyBridge
import uniffi.platform_support.AttestedKeyException.KeyException
import uniffi.platform_support.AttestedKeyType
import uniffi.platform_support.IdentifierAttestedKeyException.KeyException as IdentifierKeyException

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class AttestedKeyBridge(context: Context) : RustAttestedKeyBridge {

    override fun keyType(): AttestedKeyType = AttestedKeyType.GOOGLE

    override fun generateIdentifier(): String {
        throw KeyException("unimplemented")
    }
    
    override fun attest(identifier: String, challenge: List<UByte>): AttestationData {
        throw IdentifierKeyException("unimplemented", false)
    }
    
    override fun sign(identifier: String, payload: List<UByte>): List<UByte> {
        throw KeyException("unimplemented")
    }
    
    override fun publicKey(identifier: String): List<UByte> {
        throw KeyException("unimplemented")
    }
    
    override fun delete(identifier: String) {
        throw KeyException("unimplemented")
    }
}
