plugins {
    kotlin("jvm") version "2.1.20"
    application
}


group = "nl.ictu.edi.wallet.uiautomation"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

kotlin {
    jvmToolchain(jdkVersion = 21)
}

dependencies {
    implementation(kotlin("stdlib"))

    implementation("com.codeborne:selenide-appium:7.9.4")
    implementation("com.squareup.moshi:moshi-kotlin:1.15.2")
    implementation("io.appium:java-client:10.0.0")
    implementation("io.github.ashwithpoojary98:appium_flutterfinder_java:1.0.10")
    implementation("io.rest-assured:rest-assured:5.5.5")
    implementation("org.junit.jupiter:junit-jupiter:5.13.4")
    implementation("org.junit-pioneer:junit-pioneer:2.3.0")
    implementation("org.junit.platform:junit-platform-suite-engine:1.13.4")
    implementation("org.slf4j:slf4j-simple:2.0.17")

    implementation(platform("io.qameta.allure:allure-bom:2.29.1"))
    implementation("io.qameta.allure:allure-junit5")
    implementation("org.json:json:20250517")
    implementation("org.tomlj:tomlj:1.1.1")
}

// Test config args and default/fallback values
val testConfigMap = mapOf<String, Any>(
    "test.config.app.identifier" to "nl.ictu.edi.wallet.latest",
    "test.config.device.name" to "emulator-5554",
    "test.config.platform.name" to "Android",
    "test.config.platform.version" to 14.0,
    "test.config.remote" to false,
)

tasks.test {
    useJUnitPlatform()
}

tasks.register<Test>("smokeTest") {
    useJUnitPlatform {
        includeTags("smoke")

        // Exclude all test suites/wrappers; when using 'includeTags' this is needed to prevent
        // duplicated test executions and ensure only the actual tagged tests are run.
        exclude("suite/**")
    }
}

tasks.register<Test>("runOnAll") {
    useJUnitPlatform {
        includeTags("runonall")

        // Exclude all test suites/wrappers; when using 'includeTags' this is needed to prevent
        // duplicated test executions and ensure only the actual tagged tests are run.
        exclude("suite/**")
    }
}

// Set system properties for test config
tasks.withType<Test>().configureEach {
    testConfigMap.forEach { (key, value) ->
        systemProperty(key, System.getProperty(key, value.toString()))
    }
}
