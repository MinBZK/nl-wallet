package screen.error

import util.MobileActions

class AttributesMissingErrorScreen : MobileActions() {

    private val attributesMissingMessage = l10n.getString("missingAttributesPageTitle")

    fun attributesMissingMessageVisible() = elementWithTextVisible(attributesMissingMessage)

}


