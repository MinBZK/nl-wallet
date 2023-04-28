# `NL-Wallet Appium framework`
`This framework is for automate the Android and iOS NL-Wallet app in single code base.`


## Contents:

* [Prerequisites Installations](#prerequisites-installations)
* [Appium Setup](#appium-setup)


## Prerequisites Installations:

1. `Java Development Kit (JDK) or Java SE version 8 or later.`</b> - Install Java and set the JAVA_HOME path on your machine.
2. `Node & NPM`</b> - Download & install node from `https://nodejs.org/en/download/`.
3. `Gradle`</b> - Install Gradle.
4. `Android`</b> - Install Android Studio & set <i><b>ANDROID_HOME</b></i> path.
    -  Downloading the Android SDK
    -  Download the Android SDK tools such as
        1. Build tools
        2. Platform tools
        3. Android Emulator
        4. Create an emulator device from AVD manager
5. `iOS`</b> - Install XCode on your machine & download required iPhone/iPad simulators.
6. `Allure Report`</b> - Install Allure Report library on your machine. Please follow below link to install it on MAC.
   https://docs.qameta.io/allure/#_installing_a_commandline


## Appium Setup:

- <b>`Install Appium`</b> - to install Appium 2.0<br>
``` 
  npm install -g appium@next 
```
- <b>`xcuitest`</b> -  driver is used to run automation test on iOS devices. <br>
``` 
  appium driver install xcuitest
```
- <b>`uiautomator2`</b> - driver is used to run automation test on Android devices.<br>
``` 
  appium driver install uiautomator2
```
- <b>`Appium Doctor`</b> - which is used to see if the appium setup is correctly done or not. Run it and fix the issues.<br>
``` 
  npm install -g appium-doctor
  appium-doctor
```

## Run Automation Tests:
### Precondition:
- Run Android emulator with API level 33
- Create an apk using `cd wallet_app && flutter pub get` followed by `flutter build apk --debug`
    
### Steps
1. Make sure that you are in the correct directory: `uiautomation` where the Gradle project is located.
2. Check if you have the Gradle Wrapper script; if the Gradle Wrapper script does not exist, you can generate it 
by running: `gradle wrapper`.
3. After generating the Gradle Wrapper script, you can run the example test using the following command:
``` 
  ./gradlew test --tests uiTests.IntroductionScreenTests.verifyIntroductionScreens
```
This will run the verifyIntroductionScreens test and output the results to the console.