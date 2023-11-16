package screen.personalize

import util.MobileActions

class PersonalizeSuccessScreen : MobileActions() {

    private val screen = find.byValueKey("personalizeSuccessPage")

    private val nextButton = find.byValueKey("primaryButtonCta")

    fun visible() = isElementVisible(screen, false)

    fun clickNextButton() = clickElement(nextButton, false)
}
