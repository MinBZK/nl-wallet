package nl.rijksoverheid.edi.wallet.integration_tests

import android.Manifest
import android.bluetooth.BluetoothManager
import android.content.pm.PackageManager
import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.GrantPermissionRule
import nl.rijksoverheid.edi.wallet.platform_support.PlatformSupport
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule

abstract class WalletDeviceTest {
    @get:Rule
    val grantBluetoothPermissions: GrantPermissionRule = GrantPermissionRule.grant(
        *if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            arrayOf(Manifest.permission.BLUETOOTH_ADVERTISE, Manifest.permission.BLUETOOTH_CONNECT)
        } else {
            emptyArray()
        },
    )

    @Before
    fun setUpWalletBridge() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        assertTrue("The device must support BLE", context.packageManager.hasSystemFeature(PackageManager.FEATURE_BLUETOOTH_LE))
        assertTrue("Enable Bluetooth on the device", context.getSystemService(BluetoothManager::class.java)?.adapter?.isEnabled == true)
        // Initialized through AndroidX Startup using the host app's libraryOverride metadata.
        PlatformSupport.getInstance(context)
    }
}
