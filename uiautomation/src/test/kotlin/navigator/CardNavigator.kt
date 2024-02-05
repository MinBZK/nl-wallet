package navigator

import screen.dashboard.DashboardScreen

class CardNavigator {

    fun toScreen(screen: CardScreen) {
        if (screen > CardScreen.Dashboard) DashboardScreen().clickPidCard()
    }
}
