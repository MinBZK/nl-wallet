# Use Case 2.2 Change biometric unlock

## Overview

| Aspect                       | Description                                                                                                                                                     |
| ---------------------------- |-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user enables or disables biometric unlock in the app, confirming with their PIN when enabling biometric unlock.                                             |
| **Goal**                     | Enabling or disabling biometric unlock, whatever the user prefers.                                                                                              |
| **Preconditions**            | <ul><li>Device supports biometrics</li></ul>                                                                                                                    |
| **Postconditions**           | <ul><li>Biometric unlock is enabled OR</li><li>Biometric unlock is disabled</li></ul>                                                                           |
| **Triggered by**             | <ul><li>User selects 'Unlock with biometric' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md).</li></ul>                                                          |
| **Additional Documentation** | <ul><li>[PIN Validation](../../architecture/pin-validation.md)</li></ul>                                                                                        |
| **Possible errors**          | <ul><li>No internet</li><li>Server unreachable</li></ul>                                                                                                        |
| **Logical Test Cases**       | <ul><li>[LTC76 User disables biometrics](../logical-test-cases.md#ltc76)</li><li>[LTC77 Setup biometrics in settings](../logical-test-cases.md#ltc77)</li></ul> |

---

## Flow

| #       | Description                                                                                                                                                            | Next                                               |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| **1**   | **PRIMARY SCENARIO**                                                                                                                                                   |                                                    |
| **1.1** | **System displays screen 'Configure Biometrics'**<ul><li>Message: Unlock with Face ID/Fingerprint</li><li>Input: Enable/disable toggle</li><li>Actions: Back</li></ul> |                                                    |
| 1.1a    | User enables biometrics                                                                                                                                                | 1.2                                                |
| 1.1b    | User disables biometrics <br>&rarr; Biometrics are disabled                                                                                                            |                                                    |
| 1.1c    | User selects Back                                                                                                                                                      | End                                                |
| **1.2** | **Operating system prompts user for biometric authentication**                                                                                                         |                                                    |
| 1.2a    | User confirms with biometric authentication                                                                                                                            | 1.3                                                |
| 1.2b    | User rejects biometric authentication                                                                                                                                  | 1.1                                                |
| **1.3** | **System executes partial flow [PF2.4 Confirm a protected action](../partial-flows/PF2.4_ConfirmProtectedAction.md)**<ul><li>Not cancelable</li></ul>                                 |                                                    |
| 1.3a    | Result: Confirm                                                                                                                                                        | 1.4                                                |
| 1.3c    | Result: Back                                                                                                                                                           | 1.1                                                |
| **1.4** | **System displays screen 'wallet is secured'**<ul><li>Message: Your NL Wallet is secured</li><li>Actions: Close, To Settings, Help</li></ul>                           |                                                    |
| 1.4a    | User selects Close                                                                                                                                                     | 1.1                                                |
| 1.4b    | User selects To Settings                                                                                                                                               | Go to: [UC9.1 Show app menu](UC9.1_ShowAppMenu.md) |
| 1.4c    | User selects Help                                                                                                                                                      | Go to: [UC9.6 Get help](UC9.6_GetHelp.md)          |
<style>td {vertical-align:top}</style>