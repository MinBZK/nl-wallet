# Use Case 9.11 View revocation code in Settings

## Overview

| Aspect                       | Description                                                                                                                     |
|------------------------------|---------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | User views its existing revocation code after confirming with its remote PIN.                                                   |
| **Goal**                     | Show the revocation code so the user can keep it in a safe place and use it if they need to revoke or reset it again if needed. |
| **Preconditions**            | *None*                                                                                                                          |
| **Postconditions**           | *None*                                                                                                                          |
| **Triggered by**             | <ul><li>User selects 'About this app' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md).</li></ul>                                 |
| **Additional Documentation** | *None*                                                                                                                          |
| **Possible errors**          | *None*                                                                                                                          |
| **Logical Test Cases**       | <ul><li>[LTC71 View revocation code](../logical-test-cases.md#ltc71)</li></ul>                                                  |
 
---

## Flow

| #       | Description                                                                                                                                                                                                                         | Next                                                |
|---------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                                                                                                |                                                     |
| **1.1** | **System displays screen 'View revocation code'**<ul>><li>Message: You use this code to delete your NL Wallet from a distance.</li><li>Actions: View, Back, Help</li></ul>                                                          |                                                     |
| 1.1a    | User selects View                                                                                                                                                                                                                   | 1.2                                                 |
| 1.1b    | User selects Back                                                                                                                                                                                                                   | Back                                                |
| 1.1c    | User selects Help                                                                                                                                                                                                                   | Show placeholder 'under construction'               |
| **1.2** | **System executes partial flow [PF2.4 Confirm a protected action](../partial-flows/PF2.4_ConfirmProtectedAction.md)**<ul><li>Not cancelable</li></ul>                                                                               |                                                     |
| 1.2a    | Result: Confirm                                                                                                                                                                                                                     | 1.3                                                 |
| 1.2b    | Result: Back                                                                                                                                                                                                                        | 1.1                                                 |
| **1.3** | **System displays screen 'Revocation code'**<ul><li>Message: Write down your revocation code.</li><li>Code display: [XXXX-XXXX-XXXX-XXXX-XX] (formatted with dashes)</li><li>Actions: I have written it down, Close, Help</li></ul> | 1.6                                                 |
| 1.3a    | User selects I have written it down                                                                                                                                                                                                 | 1.1                                                 |
| 1.3b    | User selects Close                                                                                                                                                                                                                  | to: [UC9.1 Show app menu](UC9.1_ShowAppMenu.md)     |
| 1.3c    | User selects I have written it down                                                                                                                                                                                                 | Show placeholder 'under construction'               |

