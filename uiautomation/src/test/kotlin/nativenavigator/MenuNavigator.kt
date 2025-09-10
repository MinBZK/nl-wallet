package nativenavigator

import nativenavigator.screen.MenuNavigatorScreen
import nativenavigator.screen.OnboardingNavigatorScreen
import nativescreen.dashboard.DashboardScreen

class MenuNavigator {

    fun toScreen(screen: MenuNavigatorScreen, bsn: String  = "999991772") {
        // Navigate through onboarding flow
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.Dashboard, bsn)

        // Navigate to menu
        if (screen > MenuNavigatorScreen.Dashboard) DashboardScreen().clickMenuButton()
    }
}
