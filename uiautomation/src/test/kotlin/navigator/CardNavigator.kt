package navigator

import navigator.screen.CardScreen
import navigator.screen.OnboardingScreen
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen

class CardNavigator {

    fun toScreen(screen: CardScreen) {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)

        // Navigate card flow
        if (screen > CardScreen.Dashboard) DashboardScreen().clickPidCard()
        if (screen > CardScreen.CardDetail) CardDetailScreen().clickCardDataButton()
    }
}
