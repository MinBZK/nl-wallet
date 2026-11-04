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
    // Sets the built-in Kotlin version, AGP defaults to an older one
    id("org.jetbrains.kotlin.android") version "2.4.21" apply false
    id("net.razvan.jacoco-to-cobertura") version "2.0.0" apply false
}

dependencyResolutionManagement {
    // To prevent adding repositories from wallet_app/android project that also links this project
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "platform_support"

include(":test-app")
