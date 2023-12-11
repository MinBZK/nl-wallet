package screen.splash

import util.MobileActions

class SplashScreen : MobileActions() {

    private val screen = find.byValueKey("splashScreen")

    private val appTitleText = find.byText(l10n.getString("appTitle"))

    fun visible() = isElementVisible(screen, false) && isElementVisible(appTitleText, false)
}
