// Standalone entry point for native device tests, without configuring Flutter's app project.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id("com.android.application") version "9.5.0" apply false
    id("com.android.library") version "9.5.0" apply false
    id("org.jetbrains.kotlin.android") version "2.4.21" apply false
    id("net.razvan.jacoco-to-cobertura") version "2.0.0" apply false
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "wallet-integration-tests"
include(":platform_support")
project(":platform_support").projectDir = file("../../../wallet_core/wallet/platform_support/android")
