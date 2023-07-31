package config

import com.squareup.moshi.Json

data class BrowserStackCapabilities(
    @Json(name = "build") val build: String,
    @Json(name = "name") val name: String,
    @Json(name = "project") val project: String
)
