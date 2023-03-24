// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/v6.4.1/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa

import android.app.KeyguardManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Log
import nl.rijksoverheid.edi.wallet.platform_support.BuildConfig
import nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.util.DeviceUtils.isRunningOnEmulator
import uniffi.hw_keystore.KeyStoreBridge
import uniffi.hw_keystore.SigningKeyBridge
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.KeyStoreException
import java.security.NoSuchAlgorithmException
import java.security.NoSuchProviderException
import java.security.spec.ECGenParameterSpec

private const val keyStoreProvider = "AndroidKeyStore"

class ECDSAKeyStore(private val context: Context) : KeyStoreBridge {
    private val keyStore: KeyStore = KeyStore.getInstance(keyStoreProvider)

    init {
        keyStore.load(null)
        Log.d("ECDSAKeyStore", "Keystore Initialized")
    }

    private val isDeviceLocked: Boolean
        get() {
            val myKM = context.getSystemService(Context.KEYGUARD_SERVICE) as? KeyguardManager
            return myKM?.isKeyguardLocked == true
        }

    @Throws(uniffi.hw_keystore.KeyStoreException::class)
    override fun getOrCreateKey(identifier: String): SigningKeyBridge {
        try {
            if (!keyExists(identifier)) generateKey(identifier)
            val key = ECDSAKey(identifier)
            val allowSoftwareBackedKeys = isRunningOnEmulator && BuildConfig.DEBUG
            if (!key.isHardwareBacked && !allowSoftwareBackedKeys) throw KeyStoreKeyError.MissingHardwareError().keyException
            return key
        } catch (ex: Exception) {
            if (ex is uniffi.hw_keystore.KeyStoreException) throw ex
            throw KeyStoreKeyError.CreateKeyError(ex).keyException
        }
    }

    @Throws(KeyStoreException::class)
    private fun keyExists(keyAlias: String): Boolean = keyStore.containsAlias(keyAlias)

    @Throws(
        NoSuchProviderException::class,
        NoSuchAlgorithmException::class,
        IllegalStateException::class
    )
    private fun generateKey(keyAlias: String) {
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
        KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC, keyStoreProvider).apply {
            initialize(spec.build())
            generateKeyPair()
        }
    }
}