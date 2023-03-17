// Inspired by IRMAMobile: https://github.com/privacybydesign/irmamobile/blob/master/android/app/src/main/java/foundation/privacybydesign/irmamobile/irma_mobile_bridge/ECDSA.java
package nl.rijksoverheid.edi.wallet.platform_support.hw_keystore.ecdsa

import android.app.KeyguardManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Log
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

    override fun getOrCreateKey(identifier: String): SigningKeyBridge {
        if (!keyExists(identifier)) generateKey(identifier)
        return ECDSAKey(identifier)
    }

    @Throws(KeyStoreException::class)
    private fun keyExists(keyAlias: String): Boolean = keyStore.containsAlias(keyAlias)

    @Throws(
        NoSuchProviderException::class,
        NoSuchAlgorithmException::class,
        IllegalStateException::class
    )
    private fun generateKey(keyAlias: String): ByteArray {
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
        val keyPairGenerator =
            KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC, keyStoreProvider)
        keyPairGenerator.initialize(spec.build())
        val kp = keyPairGenerator.generateKeyPair()
        return kp.public.encoded
    }
}