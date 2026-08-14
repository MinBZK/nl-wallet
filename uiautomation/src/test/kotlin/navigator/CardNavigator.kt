package navigator

import navigator.screen.CardNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen
import util.MobileActions

class CardNavigator : MobileActions() {

    fun toScreen(screen: CardNavigatorScreen) {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        // Navigate card flow
        val pidDisplayName = cardMetadata.getPidDisplayName()
        Thread.sleep(SCREEN_TRANSITION_MILLIS)
        if (screen > CardNavigatorScreen.Dashboard) DashboardScreen().clickCard(pidDisplayName)
        if (screen > CardNavigatorScreen.CardDetail) CardDetailScreen().clickCardDataButton()
    }
}
