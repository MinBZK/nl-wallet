package navigator

import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import screen.dashboard.DashboardScreen

class MenuNavigator {

    fun toScreen(screen: MenuNavigatorScreen) {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard)

        // Navigate to menu
        if (screen > MenuNavigatorScreen.Dashboard) DashboardScreen().clickMenuButton()
    }
}
