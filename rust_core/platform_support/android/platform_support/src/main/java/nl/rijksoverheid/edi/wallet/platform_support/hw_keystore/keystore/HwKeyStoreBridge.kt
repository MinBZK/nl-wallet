// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore

import android.app.KeyguardManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Log
import androidx.annotation.VisibleForTesting
import nl.rijksoverheid.edi.wallet.platform_support.BuildConfig
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.PlatformSupportInitializer
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.keystore.SymmetricKey.Companion.KEY_SIZE
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.DeviceUtils.isRunningOnEmulator
import uniffi.platform_support.EncryptionKeyBridge
import uniffi.platform_support.KeyStoreBridge
import uniffi.platform_support.SigningKeyBridge
import uniffi.platform_support.initHwKeystore
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.KeyStoreException
import java.security.NoSuchAlgorithmException
import java.security.NoSuchProviderException
import java.security.spec.ECGenParameterSpec
import javax.crypto.KeyGenerator

private const val KEYSTORE_PROVIDER = "AndroidKeyStore"

/**
 * This class is automatically initialized on app start through
 * the [PlatformSupportInitializer] class.
 */
class HwKeyStoreBridge(private val context: Context) : KeyStoreBridge {
    companion object {
        @VisibleForTesting
        lateinit var bridge: KeyStoreBridge
    }

    private val keyStore: KeyStore = KeyStore.getInstance(KEYSTORE_PROVIDER)

    init {
        keyStore.load(null)
        Log.d("ECDSAKeyStore", "Keystore Initialized")
        bridge = this
        initHwKeystore(this)
    }

    private val isDeviceLocked: Boolean
        get() {
            val myKM = context.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
            return myKM?.isKeyguardLocked == true
        }

    @Throws(uniffi.platform_support.KeyStoreException::class)
    override fun getOrCreateSigningKey(identifier: String): SigningKeyBridge {
        val id = "ecdsa_$identifier"
        try {
            if (!keyExists(id)) generateSigningKey(id)
            val key = ECDSAKey(id)
            val allowSoftwareBackedKeys = isRunningOnEmulator && BuildConfig.DEBUG
            return when {
                key.isHardwareBacked -> key
                allowSoftwareBackedKeys -> key
                else -> throw KeyStoreKeyError.MissingHardwareError(key.securityLevelCompat).keyException
            }
        } catch (ex: Exception) {
            if (ex is uniffi.platform_support.KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    override fun getOrCreateEncryptionKey(identifier: String): EncryptionKeyBridge {
        val id = "aes_$identifier"
        try {
            if (!keyExists(id)) generateEncryptionKey(id)
            val key = SymmetricKey(id)
            val allowSoftwareBackedKeys = isRunningOnEmulator && BuildConfig.DEBUG
            return when {
                key.isHardwareBacked -> key
                allowSoftwareBackedKeys -> key
                else -> throw KeyStoreKeyError.MissingHardwareError(key.securityLevelCompat).keyException
            }
        } catch (ex: Exception) {
            if (ex is uniffi.platform_support.KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    @VisibleForTesting
    fun clean() = keyStore.aliases().asSequence().forEach(::deleteEntry)

    private fun deleteEntry(identifier: String) = keyStore.deleteEntry(identifier)

    @Throws(KeyStoreException::class)
    private fun keyExists(keyAlias: String): Boolean = keyStore.containsAlias(keyAlias)

    @Throws(
        NoSuchProviderException::class,
        NoSuchAlgorithmException::class,
        IllegalStateException::class
    )
    private fun generateSigningKey(keyAlias: String) {
        if (isDeviceLocked) {
            throw IllegalStateException("Key generation not allowed while device is locked")
        }
        val spec = KeyGenParameterSpec.Builder(keyAlias, KeyProperties.PURPOSE_SIGN)
            .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
            .setDigests(KeyProperties.DIGEST_SHA256)

        // setUnlockedDeviceRequired (when Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) which should work
        // throws exceptions on some devices, hence we use isDeviceLocked() for the time being
        // Issue tracker: https://issuetracker.google.com/u/1/issues/191391068
        // spec.setUnlockedDeviceRequired(true);
        val pm = context.packageManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P && pm.hasSystemFeature(PackageManager.FEATURE_STRONGBOX_KEYSTORE)) {
            spec.setIsStrongBoxBacked(true)
        }
        KeyPairGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_EC,
            KEYSTORE_PROVIDER
        ).apply {
            initialize(spec.build())
            generateKeyPair()
        }
    }

    @Throws(
        NoSuchProviderException::class,
        NoSuchAlgorithmException::class,
        IllegalStateException::class
    )
    private fun generateEncryptionKey(keyAlias: String) {
        if (isDeviceLocked) {
            throw IllegalStateException("Key generation not allowed while device is locked")
        }

        val spec = KeyGenParameterSpec.Builder(
            keyAlias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        ).setBlockModes(SymmetricKey.BLOCK_MODE)
            .setEncryptionPaddings(SymmetricKey.PADDING)
            .setKeySize(KEY_SIZE * 8 /* in bits */)
            .setUserAuthenticationRequired(false)
            .setRandomizedEncryptionRequired(true)

        // setUnlockedDeviceRequired (when Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) which should work
        // throws exceptions on some devices, hence we use isDeviceLocked() for the time being
        // Issue tracker: https://issuetracker.google.com/u/1/issues/191391068
        // spec.setUnlockedDeviceRequired(true);
        val pm = context.packageManager
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P && pm.hasSystemFeature(PackageManager.FEATURE_STRONGBOX_KEYSTORE)) {
            spec.setIsStrongBoxBacked(true)
        }
        KeyGenerator.getInstance(SymmetricKey.ALGORITHM, KEYSTORE_PROVIDER).apply {
            init(spec.build())
            generateKey()
        }
    }
}
