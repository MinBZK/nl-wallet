package config

import com.squareup.moshi.Json
import com.squareup.moshi.JsonAdapter
import com.squareup.moshi.JsonClass
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import util.EnvironmentUtil

import java.io.File

@JsonClass(generateAdapter = true)
data class TestDataConfig(
    @Json(name = "appActivity") val appActivity: String,
    @Json(name = "appPackage") val appPackage: String,
    @Json(name = "browserStackCapabilities") val browserStackCapabilities: BrowserStackCapabilities,
    @Json(name = "browserstackDevices") val browserstackDevices: List<TestDevice>,
    @Json(name = "bundleId") val bundleId: String,
    @Json(name = "server") val server: String,
    @Json(name = "sessionUrl") val sessionUrl: String,
    @Json(name = "uploadedApp") val uploadedApp: String,
    @Json(name = "device") val device: String,
    @Json(name = "localDevices") val localDevices: List<TestDevice>,
    @Json(name = "remoteOrLocal") val remoteOrLocal: RemoteOrLocal,
) {

    val defaultLocalDevice: TestDevice?
        get() = localDevices.firstOrNull { it.deviceName == device }

    val defaultRemoteDevice: TestDevice?
        get() = browserstackDevices.firstOrNull { it.deviceName == EnvironmentUtil.getVar("BROWSERSTACK_DEVICE") }
            ?: browserstackDevices.firstOrNull { it.deviceName == device }

    companion object {
        val testDataConfig = readTestDataConfig()
        val browserstackUserName = EnvironmentUtil.getVar("BROWSERSTACK_USER")
        val browserstackAccessKey = EnvironmentUtil.getVar("BROWSERSTACK_KEY")

        private fun readTestDataConfig(): TestDataConfig {
            val moshi = Moshi.Builder()
                .add(KotlinJsonAdapterFactory())
                .build()
            val jsonString = File("src/main/resources/device.conf.json").readText()
            val jsonAdapter: JsonAdapter<TestDataConfig> = moshi.adapter(TestDataConfig::class.java)
            val result = jsonAdapter.fromJson(jsonString)

            return result ?: throw IllegalStateException("device.conf.json could not be read")
        }
    }
}
