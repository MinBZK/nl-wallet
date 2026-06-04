package nl.rijksoverheid.edi.wallet

import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import dev.fluttercommunity.workmanager.LoggingDebugHandler
import dev.fluttercommunity.workmanager.WorkmanagerDebug
import io.flutter.embedding.android.FlutterFragmentActivity

class MainActivity : FlutterFragmentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Skip the Recents snapshot entirely instead of relying on FLAG_SECURE being
        // toggled in time. On Android 15 the snapshot is captured before the lifecycle
        // callbacks fire, which leaves real content visible in the app switcher.
        // setRecentsScreenshotEnabled was added in API 33 (Android 13); on older
        // versions the FLAG_SECURE toggling below remains the only mechanism.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            setRecentsScreenshotEnabled(false)
        }
        WorkmanagerDebug.setCurrent(LoggingDebugHandler())
    }

    override fun onResume() {
        super.onResume()
        window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE);
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) {
            window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
        } else {
            window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
        }
    }

    override fun onPause() {
        super.onPause()
        window.addFlags(WindowManager.LayoutParams.FLAG_SECURE);
    }
}
