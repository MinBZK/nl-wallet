package feature.personalize


import helper.GbaDataHelper
import helper.GbaDataHelper.Field.CITY
import helper.GbaDataHelper.Field.FIRST_NAME
import helper.GbaDataHelper.Field.HOUSE_NUMBER
import helper.GbaDataHelper.Field.NAME
import helper.GbaDataHelper.Field.POSTAL_CODE
import helper.GbaDataHelper.Field.STREET
import helper.TestBase
import navigator.OnboardingNavigator
import navigator.screen.OnboardingNavigatorScreen
import org.junit.jupiter.api.Assertions.assertAll
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.MethodOrderer
import org.junit.jupiter.api.TestInfo
import org.junit.jupiter.api.TestMethodOrder
import org.junitpioneer.jupiter.RetryingTest
import screen.personalize.PersonalizeConfirmPinScreen
import screen.personalize.PersonalizePidDataIncorrectScreen
import screen.personalize.PersonalizePidPreviewScreen

@TestMethodOrder(MethodOrderer.DisplayName::class)
@DisplayName("${PersonalizePidPreviewTests.USE_CASE} User confirms/rejects offered PID [${PersonalizePidPreviewTests.JIRA_ID}]")
class PersonalizePidPreviewTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 3.1"
        const val JIRA_ID = "PVW-1037"
    }

    private lateinit var personalizePidPreviewScreen: PersonalizePidPreviewScreen
    private lateinit var gbaData: GbaDataHelper

    fun setUp(testInfo: TestInfo) {
        startDriver(testInfo)
        OnboardingNavigator().toScreen(OnboardingNavigatorScreen.PersonalizePidPreview)
        personalizePidPreviewScreen = PersonalizePidPreviewScreen()
        gbaData = GbaDataHelper()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 When the PID provider offers PID data, the app displays this PID data to the user. 2 The App displays the PID data in a user friendly / human readable format. 3 The App asks the User to check whether the data is correct, and offers two buttons: confirm and reject. [$JIRA_ID]")
    fun verifyPersonalizePidPreviewScreen(testInfo: TestInfo) {
        setUp(testInfo)
        assertAll(
            { assertTrue(personalizePidPreviewScreen.visible(), "personalize pid preview screen is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(FIRST_NAME, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(NAME, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(STREET, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(CITY, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(HOUSE_NUMBER, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(gbaData.getValueByField(POSTAL_CODE, DEFAULT_BSN)), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.humanReadableCardDataVisible(DEFAULT_BSN), "human readable pid data is not visible") },
            { assertTrue(personalizePidPreviewScreen.confirmButtonsVisible(), "confirm buttons are not visible") }
        )
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 When the User confirms, the User must enter their PIN (UC2.4 Confirm a Protected action). [$JIRA_ID]")
    fun verifyAcceptPidPreview(testInfo: TestInfo) {
        setUp(testInfo)
        personalizePidPreviewScreen.clickAcceptButton()

        val personalizeConfirmPinScreen = PersonalizeConfirmPinScreen()
        assertTrue(personalizeConfirmPinScreen.visible(), "personalize confirm pin screen is not visible")
    }

    /**
     * 5. When the User has confirmed with PIN, the App continues with FEAT 'App performs PID/address issuance with PID provider'.
     * >> This requirement hard, if not impossible to be tested in an e2e setup.
     */

    /**
     * 6. When the User has entered a wrong PIN too many times (according to PIN policy), the App informs the User that the PIN was entered too many times and they should set up a new PIN. This clears the app locally and triggers UC 2.1 Setup PIN.
     * >> Test this in different scenario.
     */

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.7 When the User rejects, the App displays a support screen. [$JIRA_ID]")
    fun verifyRejectPidPreview(testInfo: TestInfo) {
        setUp(testInfo)
        personalizePidPreviewScreen.clickRejectButton()

        val personalizePidDataIncorrectScreen = PersonalizePidDataIncorrectScreen()
        assertTrue(personalizePidDataIncorrectScreen.visible(), "personalize pid data incorrect is not visible")
    }
}
