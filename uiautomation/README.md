# NL Wallet UI Automation framework

This framework is for automating the Android and iOS NL-Wallet app in a single code base using Appium and BrowserStack.

## Contents

* [Prerequisites for installation on MacOS](#prerequisites-for-installation-on-macos)
* [Appium Setup](#appium-setup)
* [Run Automation Tests](#run-automation-tests)
* [Test Annotations](#test-annotations)

## Prerequisites for installation on MacOS

1. Install Java SE Development Kit 11 or later. https://www.oracle.com/java/technologies/downloads/#java11</b>
2. Set JAVA_HOME in your environment variable:
   `export JAVA_HOME=$(/usr/libexec/java_home)
   export PATH=$JAVA_HOME/bin:$PATH`.
3. `Node & NPM`</b> - Download & install node from `https://nodejs.org/en/download/`.
4. `Gradle`</b> - Install Gradle.
5. `Android`</b> - Install Android Studio & set <i><b>ANDROID_HOME</b></i> path.
    - Downloading the Android SDK
    - Download the Android SDK tools such as
        1. Build tools
        2. Platform tools
        3. Android Emulator
        4. Create an emulator device with API version 33 from AVD manager
6. `iOS`</b> - Install XCode on your machine & download required iPhone/iPad simulators.

## Appium Setup

1. Install [Appium](https://appium.io/docs/en/2.0/quickstart/install/): `npm i --location=global appium`
2. Install [Appium Flutter Driver](https://github.com/appium-userland/appium-flutter-driver): `appium driver install --source=npm appium-flutter-driver`
3. Verify setup by running: `appium driver doctor flutter`

## Run Automation Tests

### Preconditions

1. Make sure that you are in the correct directory: `uiautomation` where the Gradle project is located.
2. See "[Customizing the test run](#customizing-test-runs)" for more info on how to customize & run test locally or remote.

### Local run

#### Preconditions

- Have an Android with support for [Application integrity](https://developer.android.com/google/play/integrity/verdicts#application-integrity-field)
- An IOS device
- First, fetch the dependencies by running `flutter pub get`
- Replace in wallet_core/wallet the files 'config-server-config.json' & 'wallet-config.json' with the files 'config-server-config.json' & 'wallet-config.json' generated by job 'wallet-config-ont' of the main pipeline
- Donwload key.properties and keystore/local-keystore.jks for Android signing and put it in wallet_app/android
- and then create an APK by executing `CONFIG_ENV=ont UNIVERSAL_LINK_BASE=https://app.example.com/deeplink/ bundle exec fastlane android build build_mode:profile file_format:apk demo_index_url:https://example.com/ universal_link_base:app.example.com`.
- and then create an IPA by executing `CONFIG_ENV="ont" UL_HOSTNAME=app.example.com UNIVERSAL_LINK_BASE="https://app.example.com/deeplink/" bundle exec fastlane ios build app_store:false build_mode:profile demo_index_url:https://example.com/ universal_link_base:app.example.com`.
- Above commands creates apps that run against the test environment.
- For IOS the value of variable ipaPath in uiautomation/src/main/kotlin/driver/LocalMobileDriver.kt needs to be changed to the location of the IPA build with the previous command.
- For IOS additional driver capabilities must be set to run the test on a real device. This can be done by filling the values in uiautomation/src/main/kotlin/driver/LocalMobileDriver.kt for the following capabilities: udid, xcodeOrgId, xcodeSigningId, updatedWDABundleId

The Appium Server will start automatically. Appium Server will handle the process of running the tests and displaying the results on the console.

### BrowserStack run

#### Precondition

- Add the following environment variables to the `~/.bash_profile` or `~/.zshrc` file:
    - `export BROWSERSTACK_USER={USERNAME}`
    - `export BROWSERSTACK_KEY={ACCESS_KEY}`
- Build the app(s):
    - Android: `bundle exec fastlane android build build_mode:profile file_format:apk` to create an APK file
    - iOS: `bundle exec fastlane ios build build_mode:profile` to create an IPA file.
- Manually upload the app(s) to BrowserStack, see [upload app from filesystem](https://www.browserstack.com/docs/app-automate/appium/upload-app-from-filesystem). By default, running tests will retrieve the latest uploaded app.

This will run the all the tests and output the test execution results on [App Automate dashboard](https://app-automate.browserstack.com/dashboard).

### Customizing test runs

The following parameters can be used to customize the test run:

1. `test.config.app.identifier`; The identifier of the app to be tested, being the package name for Android or the bundle ID for iOS.
2. `test.config.device.name`; The name of the device to be used for testing, use `emulator-5554` for local Android testing or `Google Pixel 8` for BrowserStack Android testing.
3. `test.config.platform.name`; The name of the platform to be used for testing, use `android` for local Android testing.
4. `test.config.platform.version`; The version of the platform to be used for testing, for example `14.0`.
5. `test.config.remote`; The value of this parameter should be set to `false` to run the tests locally, else `true` for BrowserStack test runs.

#### Local test run examples:

Smoke test run example:

````bash
./gradlew smokeTest
````

Full test suite run example:

````bash
./gradlew test --tests suite.FullTestSuite
    -Dtest.config.app.identifier="nl.ictu.edi.wallet.latest"
    -Dtest.config.device.name="emulator-5554"
    -Dtest.config.platform.name="Android"
    -Dtest.config.platform.version="14.0"
    -Dtest.config.remote=false
````

Remote test run example:

````bash
./gradlew test --tests suite.FullTestSuite
    -Dtest.config.app.identifier="nl.ictu.edi.wallet.latest"
    -Dtest.config.device.name="Google Pixel 8"
    -Dtest.config.platform.name="Android"
    -Dtest.config.platform.version="14.0"
    -Dtest.config.remote=true
````

## Test Annotations

JUnit 5 provides a variety of test annotations that offer capabilities for organizing and configuring tests. These annotations allow you to customize the behavior of your tests and provide additional context or information.

### @RetryingTest

#### @RetryingTest(value = n, name = "{displayName} - #{index}")

The @RetryingTest annotation allows for retrying tests that may fail due to external systems or other factors beyond the control of the code under test. This feature is particularly useful in cases where avoiding such failures is not feasible.

By applying the @RetryingTest annotation with the following attributes, you can control the behavior of the retry mechanism:

- The value attribute specifies the number of times the test will be executed before giving up.
- The name attribute determines the display name for each individual test invocation.
- The index attribute will be replaced with the current invocation index.

### @DisplayName

#### @DisplayName("UC 1.2 - Feature ticket title [PVW-1234]")

The @DisplayName enables the creation of custom names for test classes and methods. By using this annotation, you can provide more meaningful and descriptive names that accurately convey the purpose and functionality of your tests.

### @Tag

#### @Tags(Tag("smoke"), Tag("android"), Tag("ios"))

The @Tag annotation allows you to assign tags to your test classes or methods. These tags can then be used for filtering, allowing you to selectively run specific tests based on the desired criteria.

### @Suite

The @Suite annotation allows you to create test suites to execute tests spread across multiple classes and packages. A test suite is a logical grouping of tests that provides a convenient way to organize and execute related tests together.

```
@SelectPackages("feature")
@Suite
@SuiteDisplayName("Feature test suite")
object FeatureTestSuite {
}
```

The code snippet provided utilizes annotations to define a test suite called "RunTests" that includes all tests within the "feature" package. Here's what each annotation does:

- @SelectPackages("feature"): This annotation specifies that only the tests within the "feature"
  package should be included in the test suite. It acts as a filter, ensuring that only tests within the specified package are executed.
- @Suite: This annotation marks the class as a test suite. It indicates that the class is responsible for defining and executing a suite of tests rather than being a regular test class.
- @SuiteDisplayName("Run all tests"): This annotation assigns a display name to the test suite. In this case, the display name is set as "Run all tests," which provides a descriptive name for the suite, indicating that it encompasses all tests within the "feature" package.
