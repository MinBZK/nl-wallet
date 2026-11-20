package nl.rijksoverheid.edi.wallet.integration_tests

import android.util.Base64
import android.util.Log
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import java.util.UUID
import java.util.concurrent.Callable
import java.util.concurrent.FutureTask

// The same C entry points are used by the iOS wallet test bundle.
interface WalletTestLibrary : Library {
    fun interface ReaderReady : Callback {
        fun invoke(marker: Pointer)
    }

    fun wallet_close_proximity_disclosure_test(readerReady: ReaderReady): Pointer?
    fun wallet_jwt_registration_certificate_test(readerReady: ReaderReady): Pointer?
    fun wallet_registration_certificate_test(case: String, readerReady: ReaderReady): Pointer?
    fun wallet_device_test_free_error(error: Pointer)
    fun close_proximity_disclosure_test_start_qr_handover()

    companion object {
        val instance: WalletTestLibrary by lazy {
            Native.load(
                "device_integration_tests",
                WalletTestLibrary::class.java,
                mapOf(Library.OPTION_STRING_ENCODING to "UTF-8"),
            )
        }

        fun runDisclosure(registrationCase: String? = null, jwtRegistrationCertificate: Boolean = false): String? =
            onRustThread {
                val readerReady = ReaderReady { marker ->
                    // The pointer is borrowed only for this callback. Signed requests exceed logcat's
                    // per-entry limit; the host wrapper reassembles these base64 chunks before parsing.
                    val encoded = Base64.encodeToString(marker.getString(0, "UTF-8").toByteArray(), Base64.NO_WRAP)
                    val chunks = encoded.chunked(2000)
                    val id = UUID.randomUUID()
                    chunks.forEachIndexed { index, chunk ->
                        Log.i("CloseProximityTest", "CLOSE_PROXIMITY_MAC_READER_CHUNK=$id:$index:${chunks.size}:$chunk")
                    }
                }
                val error = when {
                    registrationCase != null -> instance.wallet_registration_certificate_test(registrationCase, readerReady)
                    jwtRegistrationCertificate -> instance.wallet_jwt_registration_certificate_test(readerReady)
                    else -> instance.wallet_close_proximity_disclosure_test(readerReady)
                }
                error?.let {
                    try {
                        it.getString(0, "UTF-8")
                    } finally {
                        instance.wallet_device_test_free_error(it)
                    }
                }
            }

        fun <T> onRustThread(block: () -> T): T {
            val task = FutureTask(Callable(block))
            // Leave Android's main thread available for BLE callbacks. Rust's debug-build
            // compression needs more stack than the default JVM worker provides.
            Thread(null, task, "wallet-device-test", 2L * 1024 * 1024).start()
            return task.get()
        }
    }
}
