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
7. `Allure Report`</b> - Install Allure Report library on your machine. Please follow [this link](https://docs.qameta.io/allure/#_installing_a_commandline) to install it on MAC.

## Appium Setup

1. Install [Appium](https://appium.io/docs/en/2.0/quickstart/install/): `npm i --location=global appium`
2. Install [Appium Flutter Driver](https://github.com/appium-userland/appium-flutter-driver): `appium driver install --source=npm appium-flutter-driver`
3. Install [Appium Doctor](https://github.com/appium/appium-doctor): `npm install -g appium-doctor`
4. Verify setup by running: `appium-doctor`

## Run Automation Tests

### Local run

#### Precondition

- Run Android emulator with API level 33. Emulator or devices needs to be started and should be unlocked.
- First, fetch the dependencies by running `flutter pub get`, and then create an APK by executing `flutter build apk --profile`
- For iOS use `bundle exec fastlane ios build build_mode:profile bundle_id:nl.ictu.edi.wallet.latest` to create a `.ipa` file

1. Open `device.conf.json` file in the resource directory
2. Replace the value of `device` with one of the devices listed under localDevices, such as `emulator-5554`
3. Set `remoteOrLocal` to `Local`
4. Make sure that you are in the correct directory: `uiautomation` where the Gradle project is located.
5. Check if you have the Gradle Wrapper script; if the Gradle Wrapper script does not exist, you can generate it by running: `gradle wrapper`.
6. After generating the Gradle Wrapper script, you can run the test suite using the following command: `./gradlew test --tests suite.RunTests`

The Appium Server will start automatically. Appium Server will handle the process of running the tests and displaying the results on the console.

### BrowserStack run

#### Precondition

- Add the following environment variables to the `~/.bash_profile` or `~/.zshrc` file:
    - `export BROWSERSTACK_USER={USERNAME}`
    - `export BROWSERSTACK_KEY={ACCESS_KEY}`
- Second, fetch the dependencies by running `flutter pub get`, and then create an APK by executing `flutter build apk --profile`
- For iOS use `bundle exec fastlane ios build build_mode:profile bundle_id:nl.ictu.edi.wallet.latest` to create a `.ipa` file
- To upload the app to BrowserStack, see [upload app from filesystem](https://www.browserstack.com/docs/app-automate/appium/upload-app-from-filesystem). By default, running tests will retrieve the latest uploaded app.

1. Open `device.conf.json` file in the resource directory
2. Replace the value of `device` with one of the devices listed under browserstackDevices, such as `Google Pixel 7 Pro`
3. Set `remoteOrLocal` to `Remote`
4. Make sure you are in the correct directory: `uiautomation`, where the Gradle project is located.
5. Check if you have the Gradle Wrapper script; if the Gradle Wrapper script does not exist, you can generate it by running: `gradle wrapper`.
6. After generating the Gradle Wrapper script, you can run the test suite using the following command: `./gradlew test --tests suite.RunTests`

This will run the all the tests and output the test execution results on [App Automate dashboard](https://app-automate.browserstack.com/dashboard).

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

#### @DisplayName("UC 9.3 - Verify introduction screens and privacy policy screen")

The @DisplayName enables the creation of custom names for test classes and methods. By using this annotation, you can provide more meaningful and descriptive names that accurately convey the purpose and functionality of your tests.

### @Tag

#### @Tags(Tag("smoke"), Tag("android"), Tag("ios"))

The @Tag annotation allows you to assign tags to your test classes or methods. These tags can then be used for filtering, allowing you to selectively run specific tests based on the desired criteria.

### @Suite

The @Suite annotation allows you to create test suites to execute tests spread across multiple classes and packages. A test suite is a logical grouping of tests that provides a convenient way to organize and execute related tests together.

```
@SelectPackages("uiTests")
@Suite
@SuiteDisplayName("Run all tests")
object RunTests {
}
```

The code snippet provided utilizes annotations to define a test suite called "RunTests" that includes all tests within the "uiTests" package. Here's what each annotation does:

- @SelectPackages("uiTests"): This annotation specifies that only the tests within the "uiTests"
  package should be included in the test suite. It acts as a filter, ensuring that only tests within the specified package are executed.
- @Suite: This annotation marks the class as a test suite. It indicates that the class is responsible for defining and executing a suite of tests rather than being a regular test class.
- @SuiteDisplayName("Run all tests"): This annotation assigns a display name to the test suite. In this case, the display name is set as "Run all tests," which provides a descriptive name for the suite, indicating that it encompasses all tests within the "uiTests" package.
