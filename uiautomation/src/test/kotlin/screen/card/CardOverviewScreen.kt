package screen.card

import screen.home.HomeScreen

class CardOverviewScreen : HomeScreen() {

    private val screen = find.byValueKey("cardOverviewScreen")

    fun visible() = isElementVisible(screen, false)
}
