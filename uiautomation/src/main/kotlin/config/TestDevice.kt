package config

import com.squareup.moshi.Json

data class TestDevice(
    @Json(name = "deviceName") val deviceName: String,
    @Json(name = "platformName") val platformName: String,
    @Json(name = "platformVersion") val platformVersion: String
)
