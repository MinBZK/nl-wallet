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

val browserstackAgent by configurations.creating

dependencies {
    browserstackAgent("com.browserstack:browserstack-java-sdk:latest.release")

    implementation(kotlin("stdlib"))

    implementation("com.codeborne:selenide-appium:7.9.4")
    implementation("com.squareup.moshi:moshi-kotlin:1.15.2")
    implementation("io.appium:java-client:10.0.0")
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

tasks.test {
    useJUnitPlatform {
        excludeTags("english")
    }
}

tasks.register<Test>("smokeTest") {
    useJUnitPlatform {
        includeTags("smoke")
        exclude("suite/**")
    }
}

tasks.register<Test>("smokeTestIOS") {
    useJUnitPlatform {
        includeTags("smokeIOS")
        exclude("suite/**")
    }
}

tasks.register<Test>("testEnglish") {
    useJUnitPlatform {
        includeTags("english")
        exclude("suite/**")
    }
}

tasks.register<Test>("testA11yBatch1") {
    useJUnitPlatform {
        includeTags("a11yBatch1")
        exclude("suite/**")
    }
}

tasks.register<Test>("testA11yBatch2") {
    useJUnitPlatform {
        includeTags("a11yBatch2")
        exclude("suite/**")
    }
}

tasks.withType<Test>().configureEach {
    jvmArgs("--add-modules=java.instrument")
    val testConfigMap = mapOf(
        "test.config.app.identifier" to "nl.ictu.edi.wallet.latest",
        "test.config.device.name" to "emulator-5554",
        "test.config.platform.name" to "Android",
        "test.config.platform.version" to 14.0,
        "test.config.remote" to false,
        "test.config.automation.name" to "UIAutomator2",
        "test.config.commit.sha" to "",
    )
    testConfigMap.forEach { (k, v) ->
        systemProperty(k, System.getProperty(k, v.toString()))
    }

    val toolchains = project.extensions.getByType(JavaToolchainService::class.java)
    javaLauncher.set(toolchains.launcherFor {
        languageVersion.set(JavaLanguageVersion.of(21))
    })

    if (System.getProperty("test.config.remote", "false").toBoolean()) {
        val agentProvider = CommandLineArgumentProvider {
            val jar = configurations[browserstackAgent.name].files.first {
                it.name.startsWith("browserstack-java-sdk") && it.extension == "jar"
            }
            println("Attaching BrowserStack SDK agent: ${jar.absolutePath}")
            listOf("-javaagent:${jar.absolutePath}")
        }
        jvmArgumentProviders.add(agentProvider)
    }
}
