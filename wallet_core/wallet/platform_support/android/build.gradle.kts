import net.razvan.JacocoToCoberturaPlugin
import net.razvan.JacocoToCoberturaTask

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("net.razvan.jacoco-to-cobertura")
}

android {
    namespace  = "nl.rijksoverheid.edi.wallet.platform_support"
    compileSdk = 34

    defaultConfig {
        minSdk = 24
        lint { targetSdk = 34 }
        testOptions { targetSdk = 34 }

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildFeatures {
        buildConfig = true
    }

    buildTypes {
        debug {
            enableUnitTestCoverage = true
            enableAndroidTestCoverage = true

            packaging {
                jniLibs.keepDebugSymbols += "**/*.so"
            }
        }
        // Profile is only added if the Flutter plugin is applied
        if (findByName("profile") != null) getByName("profile") {
            isMinifyEnabled = false
            isShrinkResources = false
            proguardFiles += listOf(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                file("proguard-rules.pro")
            )
        }
        release {
            isMinifyEnabled = false
            isShrinkResources = false
            proguardFiles += listOf(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                file("proguard-rules.pro")
            )
        }
    }

    // Note: When you see the following warnings:
    //
    //   warning: [options] source value 8 is obsolete and will be removed in a future release
    //   warning: [options] target value 8 is obsolete and will be removed in a future release
    //
    // That indicates that a transitive dependency still has VERSION_1_8 specified. It is emphatically
    // *not* due to the sourceCompatibility, targetCompatibility and jvmTarget settings configured below
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_11.toString()
        freeCompilerArgs += "-Xstring-concat=inline"
    }

    sourceSets {
        getByName("main") {
            // UniFFI generated bindings
            kotlin.srcDirs("../kotlin")
        }
    }
}

tasks.withType<Test>().configureEach {
    reports {
        junitXml.required = true
    }
}

tasks.named<JacocoToCoberturaTask>(JacocoToCoberturaPlugin.TASK_NAME) {
    inputFile = layout.buildDirectory.file("reports/coverage/androidTest/debug/connected/report.xml")
    outputFile = layout.buildDirectory.file("reports/coverage/androidTest/debug/connected/cobertura.xml")
}
tasks.named { it == "connectedAndroidTest" }.configureEach {
    finalizedBy(JacocoToCoberturaPlugin.TASK_NAME)
}

dependencies {
    implementation("androidx.core:core-ktx:1.9.0") // Kotlin nice-to-haves
    implementation("androidx.startup:startup-runtime:1.1.1") // Auto initialization
    implementation("com.google.android.play:integrity:1.4.0") // Play Integrity API
    implementation("net.java.dev.jna:jna:5.14.0@aar") // Java Native Access, Android Archive version
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1") // Kotlin coroutines, core library
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-play-services:1.7.3") // Kotlin coroutines for play-services

    // Test dependencies
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
    androidTestImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.8.1")
}

// Target directory for the Rust library files & bindings
val jniTargetDir = android.sourceSets.getByName("main").jniLibs.srcDirs.first()
val moduleWorkingDir = file("${project.projectDir}/../")
val bindingsTargetDir = "$moduleWorkingDir/kotlin"

// Register a task to generate Kotlin bindings
tasks.register<Exec>("cargoBuildNativeBindings") {
    executable = "$moduleWorkingDir/generate_native_bindings.sh"

    inputs.files(fileTree("$moduleWorkingDir/udl").matching { include("*.udl") })
    outputs.dir(bindingsTargetDir)
    args(listOf("kotlin") + inputs.files.map { it.name })
}

// Register tasks to build the Rust code and copy the resulting library files
enum class BuildMode { Debug, Profile, Release }
data class BuildOptions(val strip: Boolean, val args: List<String> = emptyList())
mapOf(
    BuildMode.Debug to BuildOptions(strip=false, args=listOf("--features", "hardware_integration_test")),
    BuildMode.Profile to BuildOptions(strip=true, args=listOf("--locked", "--release")),
    BuildMode.Release to BuildOptions(strip=true, args=listOf("--locked", "--release")),
).forEach { (buildMode, options) ->
    tasks.named { it == "compile${buildMode}Kotlin" }.configureEach {
        dependsOn("cargoBuildNativeBindings")
    }

    tasks.register<Exec>("cargoBuildNativeLibrary${buildMode}") {
        workingDir = moduleWorkingDir
        executable = "cargo"
        args("ndk",
            "-t", "armeabi-v7a",
            "-t", "arm64-v8a",
            "-t", "x86_64",
            "-o", jniTargetDir)
        if (!options.strip) {
            args("--no-strip")
        }
        args("build")
        args(options.args)
    }
    tasks.named { it == "merge${buildMode}NativeLibs" }.configureEach {
        dependsOn("cargoBuildNativeLibrary${buildMode}")
    }
}

tasks.register<Delete>("cleanBindings") {
    delete(bindingsTargetDir)
    doFirst {
        logger.quiet("Clean $bindingsTargetDir")
    }
}

tasks.register<Delete>("cleanJni") {
    delete(jniTargetDir)
    doFirst {
        logger.quiet("Clean $jniTargetDir")
    }
}

tasks.named("clean") {
    dependsOn("cleanBindings")
    dependsOn("cleanJni")
}
