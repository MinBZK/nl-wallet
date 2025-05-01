package nl.rijksoverheid.edi.wallet.platform_support

import android.content.Context
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.os.Build
import androidx.startup.Initializer

// Any app consuming this library can (optionally) use this key to override which .so should be loaded
private const val LIBRARY_OVERRIDE_MANIFEST_KEY =
    "nl.rijksoverheid.edi.wallet.platform_support.libraryOverride"

// The key used by the generated code [hw_keystore.kt] to check which .so should be loaded
private const val LIBRARY_OVERRIDE_PROPERTY_KEY =
    "uniffi.component.platform_support.libraryOverride"

class PlatformSupportInitializer : Initializer<PlatformSupport> {
    override fun create(context: Context): PlatformSupport {
        // Catch exception because metadata (manifest) is not available during tests.
        // Consumed because a more descriptive error is thrown if the property is not set.
        runCatching {
            val appInfo = context.packageManager.getApplicationInfoCompat(
                context.packageName,
                PackageManager.GET_META_DATA
            )
            appInfo.metaData.getString(LIBRARY_OVERRIDE_MANIFEST_KEY)?.let { libraryOverride ->
                System.setProperty(LIBRARY_OVERRIDE_PROPERTY_KEY, libraryOverride)
            }
        }
        return PlatformSupport.getInstance(context)
    }

    override fun dependencies(): List<Class<out Initializer<*>>> = emptyList()
}

private fun PackageManager.getApplicationInfoCompat(
    packageName: String,
    flags: Int = 0
): ApplicationInfo =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        getApplicationInfo(packageName, PackageManager.ApplicationInfoFlags.of(flags.toLong()))
    } else {
        @Suppress("DEPRECATION") getApplicationInfo(packageName, flags)
    }