package util

import util.SetupTestTagHandler.Companion.platformName

object Deeplink {

    const val SCHEME = "walletdebuginteraction"

    fun deeplinkToHomeScreen() {
        return deeplinkTo("deepdive#home")
    }

    private fun deeplinkTo(deepLink: String) {
        val command = if (platformName == "android") {
            "adb shell am start -a android.intent.action.VIEW -d $SCHEME://$deepLink"
        } else {
            "xcrun simctl openurl booted $SCHEME://$deepLink"
        }

        Runtime.getRuntime().exec(command).waitFor()
    }
}
