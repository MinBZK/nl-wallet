package util

import config.RemoteOrLocal
import config.TestDataConfig.Companion.testDataConfig
import org.junit.jupiter.api.TestInfo
import javax.naming.ConfigurationException

class SetupTestTagHandler {

    companion object {
        private const val ENGLISH_LANGUAGE_TAG = "english"
        private const val FRANCE_LANGUAGE_TAG = "france"

        var language: String = ""
        var locale: String = ""
        var platformName = ""

        fun handleTestTags(testInfo: TestInfo) {
            platformName = when (testDataConfig.remoteOrLocal) {
                RemoteOrLocal.Remote -> testDataConfig.defaultRemoteDevice?.platformName
                RemoteOrLocal.Local -> testDataConfig.defaultLocalDevice?.platformName
            }
                ?: throw UninitializedPropertyAccessException("Make sure 'device' in setupTestTagHandler resolves to a platformName")

            setupLanguage(testInfo)
        }

        private fun setupLanguage(testInfo: TestInfo) {
            setDutchLanguage() //Default to dutch when no language tag is set
            testInfo.tags.forEach { tag ->
                when (tag) {
                    ENGLISH_LANGUAGE_TAG -> setEnglishLanguage()
                    FRANCE_LANGUAGE_TAG -> setFranceLanguage()
                }
            }
            if (testInfo.tags.containsAll(listOf(ENGLISH_LANGUAGE_TAG, FRANCE_LANGUAGE_TAG))) {
                throw ConfigurationException("Multiple foreign language tags are not allowed.")
            }
        }

        private fun setEnglishLanguage() {
            language = "EN"
            locale = if (platformName == "android") "US" else "en_US"
        }

        private fun setFranceLanguage() {
            language = "fr"
            locale = if (platformName == "android") "FR" else "fr-FR"
        }

        private fun setDutchLanguage() {
            language = "NL"
            locale = if (platformName == "android") "NL" else "nl-NL"
        }
    }
}
