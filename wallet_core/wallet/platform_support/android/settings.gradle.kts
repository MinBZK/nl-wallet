pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

plugins {
    id("com.android.application") version "8.10.0" apply false
    id("com.android.library") version "8.10.0" apply false
    id("org.jetbrains.kotlin.android") version "2.2.0" apply false
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
