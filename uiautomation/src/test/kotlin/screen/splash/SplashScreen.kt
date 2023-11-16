package screen.splash

import util.MobileActions

class SplashScreen : MobileActions() {

    private val screen = find.byValueKey("splashScreen")

    private val appNameText = find.byText("NL Wallet")

    fun visible() = isElementVisible(screen, false)

    fun readAppNameText() = readElementText(appNameText)
}
