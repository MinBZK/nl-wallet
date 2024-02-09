package navigator

import navigator.screen.CardScreen
import navigator.screen.OnboardingScreen
import screen.dashboard.DashboardScreen

class CardNavigator {

    fun toScreen(screen: CardScreen) {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)

        // Navigate card flow
        if (screen > CardScreen.Dashboard) DashboardScreen().clickPidCard()
    }
}
