package nativenavigator

import nativenavigator.screen.CardNavigatorScreen
import nativenavigator.screen.OnboardingNavigatorScreen
import nativescreen.card.CardDetailScreen
import nativescreen.dashboard.DashboardScreen
import util.NativeMobileActions

class CardNavigator : NativeMobileActions() {

    fun toScreen(screen: CardNavigatorScreen) {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        // Navigate card flow
        val vct = cardMetadata.getPidVCT()
        if (screen > CardNavigatorScreen.Dashboard) DashboardScreen().clickCard(vct)
        if (screen > CardNavigatorScreen.CardDetail) CardDetailScreen().clickCardDataButton()
    }
}
