# Use Case 2.6 Change remote PIN

## Overview

| Aspect                       | Description                                                                                                                                                                                                                                            |
|------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user changes their PIN by first entering the current one, then choosing and confirming a new PIN that meets policy requirements.                                                                                                                   |
| **Goal**                     | Changing the remote PIN.                                                                                                                                                                                                                               |
| **Preconditions**            | <ul><li>User completed [UC3.1 Obtain PID from provider](UC3.1_ObtainPidFromProvider.md)</li><li>User completed [UC1.2 Open the app](UC1.2_OpenTheApp.md)</li></ul>                                                                                     |
| **Postconditions**           | <ul><li>User can use the new PIN in the Wallet.</li><li>User can no longer use the old PIN in the Wallet.</li></ul>                                                                                                                                    |
| **Triggered by**             | <ul><li>User selects 'Change pin' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md).</li></ul>                                                                                                                                                            |
| **Additional Documentation** | <ul><li>[PIN validation](../../architecture/use-cases/pin-validation)</li></ul>                                                                                                                                                                        |
| **Possible errors**          | *None*                                                                                                                                                                                                                                                 |
| **Logical Test Cases**       | <ul><li>[LTC56 PIN Change Happy flow](../logical-test-cases.md#ltc76)</li><li>[LTC64 PIN change fails, could not reach server](../logical-test-cases.md#ltc77)</li><li>[LTC65 PIN change fails, no internet](../logical-test-cases.md#ltc57)</li></ul> |

---

## Flow

| #       | Description                                                                                                                                           | Next                                                                    |
|---------|-------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                  |                                                                         |
| **1.1** | **System executes partial flow [PF2.4 Confirm a protected action](../partial-flows/PF2.4_ConfirmProtectedAction.md)**<ul><li>Not cancelable</li></ul> |                                                                         |
| 1.1a    | Result: Confirm                                                                                                                                       | 1.2                                                                     |
| 1.1b    | Result: Back                                                                                                                                          | Back                                                                    |
| **1.2** | **System executes partial flow [PF2.9 Setup PIN](../partial-flows/PF2.9_SetupPin.md)**                                                                |                                                                         |
| 1.2a    | Result: PIN setup succeeds                                                                                                                            | 1.3                                                                     |
| 1.2b    | Result: Back                                                                                                                                          | Back                                                                    |
| **1.3** | **System displays screen 'PIN change success'**<ul><li>Message: Success!</li><li>Actions: Dashboard, Settings</li></ul>                               |                                                                         |
| 1.3a    | User selects Dashboard                                                                                                                                | Go to: [UC7.1 Show all available cards](UC7.1_ShowAllAvailableCards.md) |
| 1.3b    | User selects Settings                                                                                                                                 | End                                                                     |
