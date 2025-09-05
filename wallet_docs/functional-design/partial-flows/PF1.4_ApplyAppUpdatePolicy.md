# Partial Flow 1.4 Apply update policy

## Overview

| Aspect                       | Description                                                                                                                                                                                   |
|------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The system validates if current app version can be used and instructs the user to update.                                                                                                     |
| **Goal**                     | Determine if app updates are available and if current app version is blocked.                                                                                                                 |
| **Preconditions**            | *None*                                                                                                                                                                                        |
| **Postconditions**           | *None*                                                                                                                                                                                        |
| **Used by**                  | <ul><li>[UC1.2 Open the app](../use-cases/UC1.2_OpenTheApp.md)</li><li>[UC2.3 Unlock the app](../use-cases/UC2.3_UnlockTheApp.md)</li><li>[PF2.4 Confirm a protected action](PF2.4_ConfirmProtectedAction.md)</li></ul> |
| **Parameters**               | *None*                                                                                                                                                                                        |
| **Possible Results**         | <ul><li>App version is allowed</li></ul>                                                                                                                                                      |
| **Additional Documentation** | *None*                                                                                                                                                                                        |
| **Possible errors**          | <ul><li>No internet</li><li>Server unreachable</li><li>Device incompatible</li></ul>                                                                                                          |
| **Logical test cases**       | <ul><li>[LTC52 App update is available](../logical-test-cases.md#ltc52)</li><li>[LTC53 Current app version is blocked](../logical-test-cases.md#ltc53)</li></ul>                              |

---

## Flow

| #       | Description                                                                                              | Next                                      |
| ------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| **1.1** | **System determines update policy for current app version**<ul><li>Duration: 0.0 - 1.5 seconds</li></ul> |                                           |
| 1.1a    | Case: Current app version is allowed                                                                     | Result: App version is allowed            |
| 1.1b    | Case: Current app version is blocked, update is available                                                | 2                                         |
| 1.1c    | Case: Current app version is blocked, no update is available                                             | 3                                         |
| 1.1d    | Error: No internet                                                                                       | Error flow: No internet                   |
| 1.1e    | Error: Server unreachable                                                                                | Error flow: Server unreachable            |
| **2**   | **WHEN APP VERSION IS BLOCKED AND UPDATE AVAILABLE**                                                     |                                           |
| **2.1** | **System displays screen 'Update needed'**<ul><li>Actions: Close, To Play/App Store</li></ul>            |                                           |
| 2.1a    | User selects Close                                                                                       | End                                       |
| 2.1b    | User selects To Play/App Store <br>&rarr; System opens Play/App Store and suspends app                   |                                           |
| **3**   | **WHEN APP VERSION IS BLOCKED AND NO UPDATE AVAILABLE**                                                  |                                           |
| **3.1** | **System displays screen 'App is blocked'**<ul><li>Actions: Help, To Play/App Store</li></ul>            |                                           |
| 3.1a    | User selects Help                                                                                        | Go to: [UC9.6 Get help](../use-cases/UC9.6_GetHelp.md) |
| 3.1b    | User selects To Play/App Store <br>&rarr; System opens Play/App Store and suspends app                   |                                           |