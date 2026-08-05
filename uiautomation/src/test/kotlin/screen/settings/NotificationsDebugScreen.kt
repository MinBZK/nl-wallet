package screen.settings

import helper.LocalizationHelper
import util.MobileActions

class NotificationsDebugScreen : MobileActions() {

    private val backButton = l10n.getString("generalWCAGBack")

    fun openPendingTab() = clickElementContainingText("Pending")

    enum class CardNotificationType {
        EXPIRES_SOON,
        EXPIRED;

        fun getDescription(l10n: LocalizationHelper, cardDisplayName: String): String {
            val cardExpiresSoonNotificationDescriptionKey = "cardExpiresSoonNotificationDescription"
            val cardExpiredNotificationDescriptionKey = "cardExpiredNotificationDescription"

            return when (this) {
                EXPIRES_SOON -> l10n.getPluralString(cardExpiresSoonNotificationDescriptionKey, 7,
                    mapOf("card" to cardDisplayName, "days" to "7"))
                EXPIRED -> l10n.getString(cardExpiredNotificationDescriptionKey).replace("{card}", cardDisplayName)
            }
        }
    }

    fun isNotificationVisible(cardDisplayName: String, type: CardNotificationType): Boolean {
        return elementWithTextVisible(type.getDescription(l10n, cardDisplayName))
    }

    private fun getDebugValue(cardDisplayName: String, type: CardNotificationType, prefix: String): String {
        val siblingText = type.getDescription(l10n, cardDisplayName)
        val descElement = scrollToElementWithText(siblingText)
        // Only scroll down when the description is in the lower half of the screen.
        // When the card is near the top (e.g. first item on a freshly-opened screen), the
        // sub-items are already visible below it. Scrolling would push the description off-screen
        // above, removing it from the accessibility tree and breaking the sibling XPath filter.
        if (descElement.location.y > driver.manage().window().size.height / 2) {
            scrollDown(200)
        }
        val element = findElementByPartialTextAndPartialSiblingText(prefix, siblingText)
        return getElementText(element).removePrefix(prefix)
    }

    fun getCardNotificationID(cardDisplayName: String, type: CardNotificationType): String =
        getDebugValue(cardDisplayName, type, "id: ")

    fun getCardNotificationChannel(cardDisplayName: String, type: CardNotificationType): String =
        getDebugValue(cardDisplayName, type, "channel: ")

    fun getCardNotificationTimer(cardDisplayName: String, type: CardNotificationType): String =
        getDebugValue(cardDisplayName, type, "notifyAt: ")

    fun clickBackButton() = clickElementWithText(backButton)
}
