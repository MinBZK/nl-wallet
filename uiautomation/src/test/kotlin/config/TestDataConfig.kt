package config

import com.squareup.moshi.Json
import com.squareup.moshi.JsonAdapter
import com.squareup.moshi.JsonClass
import com.squareup.moshi.Moshi
import com.squareup.moshi.kotlin.reflect.KotlinJsonAdapterFactory
import java.io.File

@JsonClass(generateAdapter = true)
data class TestDataConfig(
    @Json(name = "appActivity") val appActivity: String,
    @Json(name = "appPackage") val appPackage: String,
    @Json(name = "browserStackCapabilities") val browserStackCapabilities: BrowserStackCapabilities,
    @Json(name = "browserstackDevices") val browserstackDevices: List<BrowserstackDevice>,
    @Json(name = "bundleId") val bundleId: String,
    @Json(name = "server") val server: String,
    @Json(name = "sessionUrl") val sessionUrl: String,
    @Json(name = "uploadedApp") val uploadedApp: String,
    @Json(name = "device") val device: String,
    @Json(name = "localDevices") val localDevices: List<LocalDevice>,
    @Json(name = "remoteOrLocal") val remoteOrLocal: RemoteOrLocal,
) {

    enum class RemoteOrLocal {
        remote,
        local
    }

    val defaultLocalDevice: LocalDevice?
        get() = localDevices.firstOrNull { it.deviceName == device }

    val defaultRemoteDevice: BrowserstackDevice?
        get() = browserstackDevices.firstOrNull { it.deviceName == getEnvVar("BROWSERSTACK_DEVICE") }
            ?: browserstackDevices.firstOrNull { it.deviceName == device }

    data class BrowserStackCapabilities(
        @Json(name = "build") val build: String,
        @Json(name = "name") val name: String,
        @Json(name = "project") val project: String
    )

    data class BrowserstackDevice(
        @Json(name = "deviceName") val deviceName: String,
        @Json(name = "platformVersion") val platformVersion: String,
        @Json(name = "platformName") val platformName: String
    )

    data class LocalDevice(
        @Json(name = "deviceName") val deviceName: String,
        @Json(name = "platformName") val platformName: String,
        @Json(name = "platformVersion") val platformVersion: String,
        @Json(name = "udid") val udid: String
    )

    companion object {
        val testDataConfig = readTestDataConfig()
        val browserstackUserName = getEnvVar("BROWSERSTACK_USER")
        val browserstackAccessKey = getEnvVar("BROWSERSTACK_KEY")

        private fun readTestDataConfig(): TestDataConfig {
            val moshi = Moshi.Builder()
                .add(KotlinJsonAdapterFactory())
                .build()
            val jsonString = File("src/main/resources/device.conf.json").readText()
            val jsonAdapter: JsonAdapter<TestDataConfig> = moshi.adapter(TestDataConfig::class.java)
            val result = jsonAdapter.fromJson(jsonString)

            return result ?: throw IllegalStateException("device.conf.json could not be read")
        }

        private fun getEnvVar(name: String): String {
            return System.getenv(name) ?: ""
        }
    }
}
