import java.util.Properties
import java.util.Base64

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

fun loadProperties(name: String): Map<String, String> {
    val file = rootProject.file(name)
    return if (file.exists()) {
        val props = Properties()
        file.inputStream().use { props.load(it) }
        // Cast to String (Properties predates generics, but key and value are always String)
        props.map { it.key as String to it.value as String }.toMap()
    } else {
        emptyMap()
    }
}

val keystoreProperties = loadProperties("key.properties")
val signingConfigName = if (keystoreProperties["storeFile"] != null) "release" else "debug"

val dartEnvironmentVariables = if (project.hasProperty("dart-defines")) {
    project.property("dart-defines").let { defines ->
        check(defines is String) { "dart-defines should be String" }
        defines.split(",").associate { entry ->
            val pair =
                Base64.getDecoder().decode(entry).toString(Charsets.UTF_8).split("=", limit = 2)
            check(pair.size == 2) { "Got dart define without =" }
            pair[0] to pair[1]
        }
    }
} else {
    mapOf()
}

val ndkTargets = System.getenv("ANDROID_NDK_TARGETS")?.split(' ')
    ?: listOf("armeabi-v7a", "arm64-v8a", "x86_64")

class ULIntentFilter(
    val autoVerify: Boolean,
    val host: String,
    val pathPrefix: String,
    val scheme: String,
)

// Universal / deep links config
val ulIntentFilter =
    dartEnvironmentVariables["UL_HOSTNAME"]?.takeIf { it.isNotEmpty() }?.let { ulHostname ->
        ULIntentFilter(
            autoVerify = true,
            host = ulHostname,
            pathPrefix = "/deeplink",
            scheme = "https"
        )
    } ?: ULIntentFilter(
        autoVerify = false,
        host = "*",
        pathPrefix = "",
        scheme = "walletdebuginteraction"
    )

android {
    namespace = "nl.rijksoverheid.edi.wallet"
    compileSdk = 35
    // Use NDK r28b to get 16kB page size
    ndkVersion = "28.1.13356709"

    // Note: When using flutter >= 3.27.1 with Java 21, you will see the
    // following (harmless) warnings:
    //
    //   warning: [options] source value 8 is obsolete and will be removed in a future release
    //   warning: [options] target value 8 is obsolete and will be removed in a future release
    //
    // This will be fixed in a future flutter version, see:
    // https://github.com/flutter/flutter/issues/156111
    //
    // The above warnings are not due to these jvmTarget, sourceCompatibility,
    // and targetCompatibility options, but due to the flutter gradle plugin.
    // When the above-linked issue is merged and released, the above warnings
    // should disappear.
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_11.toString()
        freeCompilerArgs = listOf("-Xstring-concat=inline")
    }

    packaging {
        // Exclude the platform_support.so files that are added by the
        // platform_support module, as this code is also in libwallet_core.so
        jniLibs {
            excludes += "**/libplatform_support.so"
        }
    }

    defaultConfig {
        applicationId = System.getenv("APPLICATION_ID") ?: "nl.ictu.edi.wallet.latest"
        // You can update the following values to match your application needs.
        // For more information, see: https://docs.flutter.dev/deployment/android#reviewing-the-build-configuration.
        minSdk = 24
        targetSdk = 34
        versionCode = flutter.versionCode
        versionName = flutter.versionName

        manifestPlaceholders["appName"] = System.getenv("APP_NAME") ?: "NL Wallet"

        // Set universal & deep links intent-filter placeholders
        manifestPlaceholders["ulIntentFilterAutoVerify"] = ulIntentFilter.autoVerify
        manifestPlaceholders["ulIntentFilterHost"] = ulIntentFilter.host
        manifestPlaceholders["ulIntentFilterPathPrefix"] = ulIntentFilter.pathPrefix
        manifestPlaceholders["ulIntentFilterScheme"] = ulIntentFilter.scheme
    }

    signingConfigs {
        create("release") {
            keyAlias = keystoreProperties["keyAlias"]
            keyPassword = keystoreProperties["keyPassword"]
            storeFile = keystoreProperties["storeFile"]?.let { file("../$it") }
            storePassword = keystoreProperties["storePassword"]
        }
    }

    buildFeatures {
        buildConfig = true
    }

    // Debug and Profile builds use release keys if they're available, auto-available debug keys if not.
    // Release build needs release keys (i.e., wallet_app/android/key.properties, wallet_app/android/keystore/local-keystore.jks).
    buildTypes {
        debug {
            signingConfig = signingConfigs.getByName(signingConfigName)
            ndk {
                abiFilters += ndkTargets
            }
        }
        getByName("profile") {
            signingConfig = signingConfigs.getByName(signingConfigName)
            ndk {
                abiFilters += ndkTargets
            }
        }
        release {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = true
            proguardFiles += listOf(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                file("proguard-rules.pro")
            )
            ndk {
                abiFilters += ndkTargets
            }
        }
    }
}

flutter {
    source = "../.."
}

dependencies {
    implementation("net.java.dev.jna:jna:5.7.0@aar") // Java Native Access

    implementation(project(path = ":platform_support"))
}

// Target directory for the Rust library files
val jniTargetDir = android.sourceSets.getByName("main").jniLibs.srcDirs.first()

// Register tasks to build the Rust code and copy the resulting library files
enum class BuildMode { Debug, Profile, Release }
data class BuildOptions(val args: List<String> = emptyList())
mapOf(
    BuildMode.Debug to BuildOptions(),
    BuildMode.Profile to BuildOptions(args = listOf("--locked", "--release")),
    BuildMode.Release to BuildOptions(args = listOf("--locked", "--release")),
).forEach { (buildMode, options) ->
    tasks.register<Exec>("cargoBuildNativeLibrary${buildMode}") {
        workingDir("../../../wallet_core")

        // Build the Rust code (wallet_core)
        executable = "cargo"
        args("ndk")
        args(ndkTargets.flatMap { listOf("-t", it) })
        args("-o", jniTargetDir)
        args("--no-strip")
        args("build", "-p", "flutter_api")
        args(options.args)
        if (dartEnvironmentVariables["ALLOW_INSECURE_URL"] == "true") {
            args("--features", "wallet/allow_insecure_url")
        }
    }
    tasks.named { it == "merge${buildMode}NativeLibs" }.configureEach {
        dependsOn("cargoBuildNativeLibrary${buildMode}")
    }
}

// Ensure non-debug keys for release builds
tasks.register("checkForReleaseKeys") {
    doFirst {
        if (signingConfigName != "release") {
            throw GradleException("Cannot do a release build with non-release keys")
        }
    }
}
tasks.named { it == "signReleaseBundle" }.configureEach {
    dependsOn("checkForReleaseKeys")
}

tasks.register<Delete>("cleanJni") {
    delete(jniTargetDir)
    doFirst {
        logger.quiet("Clean $jniTargetDir")
    }
}

tasks.named("clean") {
    dependsOn("cleanJni")
}
