# Use Case 1.2 Open the app

## Overview

| Aspect                       | Description                                                                                                                                                                                                                                                                                     |
| ---------------------------- |-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user opens the app manually or follows a universal link. The system checks the app version is valid, or forces the user to update.<br>When valid, the system redirects to app introduction or app unlock.                                                                                   |
| **Goal**                     | Opening the app.                                                                                                                                                                                                                                                                                |
| **Preconditions**            | <ul><li>App is installed</li><li>App version is allowed</li></ul>                                                                                                                                                                                                                               |
| **Postconditions**           | *None*                                                                                                                                                                                                                                                                                          |
| **Triggered by**             | <ul><li>User launches or resumes the app manually.</li><li>User launches or resumes the app by following a universal link.</li></ul>                                                                                                                                                            |
| **Additional Documentation** | <ul><li>[App Startup](../../architecture/mobile-app-startup.md)</li><li>[iOS App Activations](https://developer.apple.com/documentation/xcode/reducing-your-app-s-launch-time)</li><li>[Android App startup time](https://developer.android.com/topic/performance/vitals/launch-time)</li></ul> |
| **Possible errors**          | <ul><li>No internet</li><li>Server unreachable</li></ul>                                                                                                                                                                                                                                        |
| **Logical test cases**       | <ul><li>[LTC42 Open closed app](../logical-test-cases.md#ltc42)</li><li>[LTC43 Open app via universal link](../logical-test-cases.md#ltc43)</li><li>[LTC54 Wallet not created when universal link is invoked](../logical-test-cases.md#ltc54)</li></ul>                                         |

---

## Flow

| #       | Description                                                                                                       | Next                                                       |
|---------|-------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                              |                                                            |
| **1.1** | **System determines app startup state**                                                                           |                                                            |
| 1.1a    | Case: App launches (iOS) or starts cold or warm (Android)<sup>1</sup>                                             | 2                                                          |
| 1.1b    | Case: App resumes (iOS) or starts hot (Android)<sup>2</sup>                                                       | 3                                                          |
| **2**   | **WHEN APP LAUNCHES**                                                                                             |                                                            |
| **2.1** | **System displays screen 'Splash screen'**                                                                        |                                                            |
| 2.1a    | Event: 1 second passes                                                                                            | 2.2                                                        |
| **2.2** | **System executes partial flow [PF1.4 Apply update policy](../partial-flows/PF1.4_ApplyAppUpdatePolicy.md)**      |                                                            |
| 2.2a    | Result: App version is allowed                                                                                    | 2.3                                                        |
| **2.3** | **System determines whether remote PIN is set up**                                                                |                                                            |
| 2.3a    | Case: User has setup a remote PIN                                                                                 | Go to: [UC2.3 Unlock the app](UC2.3_UnlockTheApp.md)       |
| 2.3b    | Case: User has not setup a remote PIN                                                                             | Go to: [UC1.1 Introduce the app](UC1.1_IntroduceTheApp.md) |
| **3**   | **WHEN APP RESUMES**                                                                                              |                                                            |
| **3.1** | **System determines whether remote PIN is set up**                                                                |                                                            |
| 3.1a    | Case: User has setup a remote PIN                                                                                 | 3.2                                                        |
| 3.1b    | Case: User has not setup a remote PIN                                                                             | Resume<sup>3</sup>                                         |
| **3.2** | **System determines whether app is locked**                                                                       |                                                            |
| 3.2a    | Case: Wallet is unlocked                                                                                          | 3.3                                                        |
| 3.2b    | Case: Wallet is locked                                                                                            | Go to: [UC2.3 Unlock the app](UC2.3_UnlockTheApp.md)       |
| **3.3** | **System determines whether user has obtained PID**                                                               |                                                            |
| 3.3a    | Case: User has obtained PID                                                                                       | 3.4                                                        |
| 3.3a    | Case: User has not obtained PID                                                                                   | Resume<sup>3</sup>                                         |
| **3.4** | **System executes partial flow [PF2.7 Resolve a universal link](../partial-flows/PF2.7_ResolveUniversalLink.md)** |                                                            |
| 3.4a    | Result: No (valid) universal link                                                                                 | Resume<sup>3</sup>                                         |
<div class="table-notes">1. We use the iOS terminology 'launch' to that no process was active when opening the app. In Android, this matches the 'cold startup' or 'warm startup' processes. Even without an active process, some resources may already be loaded into memory.</div>
<div class="table-notes">2. We use the iOS terminology 'resume' to that the app process was already active when opening the app. In Android, this matches the 'hot startup' process.</div>
<div class="table-notes">3. Resume the state when the app backgrounded/suspended.</div>