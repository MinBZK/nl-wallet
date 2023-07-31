package util

import util.SetupTestTagHandler.Companion.platformName

object Deeplink {

    private const val SCHEME = "walletdebuginteraction"

    fun deeplinkToHomeScreen() {
        return deeplinkTo("deepdive#home")
    }

    private fun deeplinkTo(deeplink: String) {
        val command = if (platformName == "android") {
            "adb shell am start -a android.intent.action.VIEW -d $SCHEME://$deeplink"
        } else {
            "xcrun simctl openurl booted $SCHEME://$deeplink"
        }

        Runtime.getRuntime().exec(command).waitFor()
    }
}
