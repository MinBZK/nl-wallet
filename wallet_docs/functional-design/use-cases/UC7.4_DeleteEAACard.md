# Use Case 7.4 Delete EAA card

## Overview

| Aspect                       | Description                                                                                                                                                                                 |
|------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user deletes an EAA card from their wallet by confirming this with their PIN to complete the process. |
| **Goal**                     | The user delete an EAA card from NL Wallet in a safe way.                                                                                                                                   |
| **Preconditions**            | *None*                                                                                                                                                                                      |
| **Postconditions**           | <ul><li>Card is no longer available in the wallet</li><li>Card deleted event is added in the history.</li></ul>                                                                             |
| **Triggered by**             | <ul><li>User selects 'Delete card' in [UC7.2 Show card details](UC7.2_ShowCardDetails.md)</li></ul>                                                                                         |
| **Additional Documentation** | *None*                                                                                                                                                                                      |
| **Possible errors**          | *None*                                                                                                                                                                                      |
| **Logical test cases**       | <ul><li>[LTC79 Delete card](../logical-test-cases.md#ltc79)</li></ul>                                                                                                                       |

---

## Flow

| #       | Description                                                                                                                                                             | Next                                                                   |
|---------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                                    |                                                                        |
| **1.1** | **System displays prompt 'Do you want to delete [card_name] card'**<ul><li>Message: The card will be permanently deleted</li><li>Actions: Cancel, Yes, delete</li></ul> |                                                                        |
| 1.1a    | User selects Yes, delete                                                                                                                                                | 1.2                                                                    |
| 1.1b    | User selects Cancel                                                                                                                                                     | Go to: [UC7.2 Show card details](UC7.2_ShowCardDetails.md)             |
| **1.2** | **System executes partial flow [PF2.4 Confirm a protected action](../partial-flows/PF2.4_ConfirmProtectedAction.md)**<ul><li>Cancelable</li></ul>                       |                                                                        |
| 1.2a    | Result: Confirm                                                                                                                                                         | 1.3                                                                    |
| 1.2b    | Result: Cancel                                                                                                                                                          | Go to: [UC7.2 Show card details](UC7.2_ShowCardDetails.md)             |
| 1.2c    | Result: Back                                                                                                                                                            | Go to: [UC7.2 Show card details](UC7.2_ShowCardDetails.md)             |
| **1.3** | **System displays screen '[Card_name] has been deleted'**<ul><li>Message: You can still see previous activities</li><li>Actions: To my overview, Close, Help</li></ul>  |                                                                        |
| 1.3a    | User selects To my overview                                                                                                                                             | Go to: [UC7.1 Show all available cards](./UC7.1_ShowAllAvailableCards) |
| 1.3b    | User selects Close                                                                                                                                                      | Go to: [UC7.1 Show all available cards](./UC7.1_ShowAllAvailableCards) |
| 1.3c    | User selects Help                                                                                                                                                       | Show placeholder 'under construction'                                  |
