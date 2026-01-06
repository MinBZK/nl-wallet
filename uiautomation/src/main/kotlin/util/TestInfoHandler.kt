package util

import data.TestConfigRepository.Companion.testConfig
import org.junit.jupiter.api.TestInfo
import javax.naming.ConfigurationException

class TestInfoHandler {

    companion object {
        private const val ENGLISH_LANGUAGE_TAG = "english"
        private const val FRANCE_LANGUAGE_TAG = "france"

        val platformName = testConfig.platformName

        lateinit var sessionName: String
        lateinit var language: String
        lateinit var locale: String

        fun processTestInfo(testInfo: TestInfo) {
            sessionName = testInfo.displayName

            setupLanguage(testInfo.tags)
        }

        private fun setupLanguage(tags: Set<String>) {
            if (EnvironmentUtil.getVar("ENABLE_BROWSERSTACK_A11Y_CHECKS") == "true") {
                setEnglishLanguage()
                return
            }
            when {
                tags.contains(ENGLISH_LANGUAGE_TAG) -> setEnglishLanguage()
                tags.contains(FRANCE_LANGUAGE_TAG) -> setFranceLanguage()
                tags.containsAll(listOf(ENGLISH_LANGUAGE_TAG, FRANCE_LANGUAGE_TAG)) ->
                    throw ConfigurationException("Multiple foreign language tags are not allowed.")
                else -> setDutchLanguage() // Default to Dutch when no language tag is set
            }
        }

        private fun setEnglishLanguage() {
            language = "en"
            locale = if (platformName == "android") "US" else "en_US"
        }

        private fun setFranceLanguage() {
            language = "fr"
            locale = if (platformName == "android") "FR" else "fr-FR"
        }

        private fun setDutchLanguage() {
            language = "nl"
            locale = if (platformName == "android") "NL" else "nl-NL"
        }
    }
}
