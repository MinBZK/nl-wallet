package util

import data.TestConfigRepository.Companion.testConfig
import domain.Platform
import org.junit.jupiter.api.TestInfo
import javax.naming.ConfigurationException

class TestInfoHandler {

    companion object {
        private const val ENGLISH_LANGUAGE_TAG = "english"
        private const val FRANCE_LANGUAGE_TAG = "france"

        val platform = testConfig.platform

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
            locale = if (platform == Platform.ANDROID) "US" else "en_US"
        }

        private fun setFranceLanguage() {
            language = "fr"
            locale = if (platform == Platform.ANDROID) "FR" else "fr-FR"
        }

        private fun setDutchLanguage() {
            language = "nl"
            locale = if (platform == Platform.ANDROID) "NL" else "nl-NL"
        }
    }
}
