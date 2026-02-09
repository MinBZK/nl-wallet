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
        val element = findElementByPartialTextAndPartialSiblingText(prefix, siblingText)
        val elementText = getElementText(element)
        return elementText.removePrefix(prefix)
    }

    fun getCardNotificationID(cardDisplayName: String, type: CardNotificationType): String {
        scrollToElementWithText(type.getDescription(l10n, cardDisplayName))
        scrollDown(200)
        return getDebugValue(cardDisplayName, type, "id: ")
    }

    fun getCardNotificationChannel(cardDisplayName: String, type: CardNotificationType): String {
        scrollToElementWithText(type.getDescription(l10n, cardDisplayName))
        scrollDown(200)
        return getDebugValue(cardDisplayName, type, "channel: ")
    }


    fun getCardNotificationTimer(cardDisplayName: String, type: CardNotificationType): String {
        scrollToElementWithText(type.getDescription(l10n, cardDisplayName))
        scrollDown(200)
        return getDebugValue(cardDisplayName, type, "notifyAt: ")
    }

    fun clickBackButton() = clickElementWithText(backButton)
}
