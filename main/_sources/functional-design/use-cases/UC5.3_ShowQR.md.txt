# Use Case 5.3 Show QR

## Overview

| Aspect                       | Description                                                                                                            |
|------------------------------|------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user shows the QR to a third party reader, to initiate a close proximity sharing session.                          |
| **Goal**                     | Starting a close proximity sharing session.                                                                            |
| **Preconditions**            | *None*                                                                                                                 |
| **Postconditions**           | *None*                                                                                                                 |
| **Triggered by**             | <ul><li>User selects 'Show your QR code' in [UC7.1 Show all available cards](UC7.1_ShowAllAvailableCards.md)</li></ul> |
| **Additional documentation** | *None*                                                                                                                 |
| **Possible errors**          | *None*                                                                                                                 |
| **Logical test cases**       | <ul><li>[LTC79 Close proximity data sharing](../logical-test-cases.md#ltc79)</li></ul>                                 |

---

## Flow

| #       | Description                                                                                                                                        | Next                                                                   |
|---------|----------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                               |                                                                        |
| **1.1** | **System determines whether Bluetooth permissions were granted**                                                                                   |                                                                        |
| 1.1a    | Case: Bluetooth permissions are granted                                                                                                            | 1.3                                                                    |
| 1.1b    | Case: Bluetooth permissions are not granted                                                                                                        | 1.2                                                                    |
| 1.1c    | Case: Bluetooth permissions have been rejected before                                                                                              | 2                                                                      |
| 1.1c    | Case: Bluetooth is disabled                                                                                                                        | 2                                                                      |
| **1.2** | **System asks permission to use Bluetooth**                                                                                                        |                                                                        |
| 1.2a    | User grants permission                                                                                                                             | 1.3                                                                    |
| 1.2b    | User rejects permission                                                                                                                            | 2                                                                      |
| **1.3** | **System displays screen 'Show your QR code'**<ul><li>Actions: Center QR code on screen, Back, Help, Would you like to report a problem?</li></ul> |                                                                        |
| 1.3a    | User scans Center QR code on screen                                                                                                                | 1.4                                                                    |
| 1.3b    | User selects Back                                                                                                                                  | Go to: [UC7.1 Show all available cards](./UC7.1_ShowAllAvailableCards) |
| 1.3c    | User selects Help                                                                                                                                  | Show placeholder 'under construction'                                  |
| 1.3d    | User selects Would you like to report a problem?                                                                                                   | Show placeholder 'under construction'                                  |
| 1.3e    | Verifier scan QR                                                                                                                                   | Go to: [UC5.1 Share data](./UC5.1_ShareDataWithRP)                     |
| **1.4** | **System displays modal with centered QR code**<ul><li>Actions: Close</li></ul>                                                                    |                                                                        |
| 1.4a    | User selects Close                                                                                                                                 | 1.3                                                                    |
| 1.4b    | Verifier scan QR                                                                                                                                   | Go to: [UC5.1 Share data](./UC5.1_ShareDataWithRP)                     |
| **2**   | **WHEN NO BLUETOOTH PERMISSION**                                                                                                                   |                                                                        |
| **2.1** | **System displays prompt 'Allow NL Wallet to find Bluetooth devices.'**<ul><li>Actions: Close, Open Settings</li></ul>                             |                                                                        |
| 2.1a    | User selects Close                                                                                                                                 | End                                                                    |
| 2.1b    | User selects Open Settings<br>&rarr; System opens OS settings and suspends app                                                                     |                                                                        |
| **3**   | **WHEN BLUETOOTH PERMISSION IS DISABLED**                                                                                                          |                                                                        |
| **3.1** | **System displays screen 'Turn on Bluetooth first'**<ul><li>Actions: Close, Help, Turn on Bluetooth (Android only)</li></ul>                       |                                                                        |
| 3.1a    | User selects Close                                                                                                                                 | Go to: [UC7.1 Show all available cards](./UC7.1_ShowAllAvailableCards) |
| 3.1b    | User selects Help                                                                                                                                  | Show placeholder 'under construction'                                  |
| 3.1c    | User selects Turn on Bluetooth                                                                                                                     | Show placeholder 'under construction'                                  |
| **3.2** | **System asks permission to turn on Bluetooth**                                                                                                    |                                                                        |
| 3.2a    | User grants permission                                                                                                                             | 1.3                                                                    |
| 3.2b    | User rejects permission                                                                                                                            | 3.1                                                                    |
