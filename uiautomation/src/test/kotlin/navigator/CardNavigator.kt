package navigator

import navigator.screen.CardNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen

class CardNavigator {

    fun toScreen(screen: CardNavigatorScreen, vct: String = "urn:eudi:pid:nl:1") {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        // Navigate card flow
        if (screen > CardNavigatorScreen.Dashboard) DashboardScreen().clickCard(vct)
        if (screen > CardNavigatorScreen.CardDetail) CardDetailScreen().clickCardDataButton()
    }
}
