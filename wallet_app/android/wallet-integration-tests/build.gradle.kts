import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    id("com.android.application")
}

val ndkTargets = System.getenv("ANDROID_NDK_TARGETS")?.split(' ')
    ?: listOf("armeabi-v7a", "arm64-v8a", "x86_64")
val rustJniDirectory = layout.buildDirectory.dir("generated/jniLibs")

android {
    namespace = "nl.rijksoverheid.edi.wallet.integration_tests"
    compileSdk = 36
    ndkVersion = "28.1.13356709"

    defaultConfig {
        applicationId = "nl.rijksoverheid.edi.wallet.integration_tests"
        minSdk = 29
        targetSdk = 36
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        testInstrumentationRunnerArguments["timeout_msec"] = "180000"
        ndk {
            abiFilters += ndkTargets
        }
    }

    installation {
        installOptions += "-g"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets.getByName("main").jniLibs.directories += rustJniDirectory.get().asFile.path
    packaging {
        jniLibs {
            // UniFFI and the wallet scenarios must share the same Rust PlatformSupport singleton.
            excludes += "**/libplatform_support.so"
        }
    }
}

androidComponents {
    beforeVariants(selector().withBuildType("release")) {
        it.enable = false
    }
}

kotlin {
    compilerOptions {
        jvmTarget = JvmTarget.JVM_17
    }
}

dependencies {
    implementation(project(":platform_support"))
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test:rules:1.6.1")
    androidTestImplementation("net.java.dev.jna:jna:5.14.0@aar")
}

val cargoBuildWalletTests = tasks.register<Exec>("cargoBuildWalletTests") {
    workingDir = file("../../../wallet_core")
    executable = "cargo"
    args("ndk")
    args(ndkTargets.flatMap { listOf("-t", it) })
    args("-o", rustJniDirectory.get().asFile)
    args("build", "--locked", "--package", "device_integration_tests", "--lib")
}

tasks.named { it == "mergeDebugNativeLibs" }.configureEach {
    dependsOn(cargoBuildWalletTests)
}
