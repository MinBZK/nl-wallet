package navigator

import navigator.screen.MenuNavigatorScreen
import navigator.screen.OnboardingNavigatorScreen
import screen.dashboard.DashboardScreen

class MenuNavigator {

    fun toScreen(screen: MenuNavigatorScreen, bsn: String  = "999991772") {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard, bsn)

        // Navigate to menu
        if (screen > MenuNavigatorScreen.Dashboard) DashboardScreen().clickMenuButton()
    }
}
