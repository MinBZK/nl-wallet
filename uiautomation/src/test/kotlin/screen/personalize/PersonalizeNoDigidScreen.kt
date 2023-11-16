package screen.personalize

import util.MobileActions

class PersonalizeNoDigidScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeNoDigidScreen")

    private val applyForDigidButton = find.byValueKey("applyForDigidCta")

    fun visible() = isElementVisible(screen)

    fun clickApplyForDigidButton() = clickElement(applyForDigidButton)

    fun switchToWebView() = switchToWebViewContext()
}
