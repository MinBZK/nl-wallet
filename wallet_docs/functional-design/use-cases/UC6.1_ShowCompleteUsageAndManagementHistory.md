# Use Case 6.1 Show complete history

## Overview

| Aspect                       | Description                                                                                                                                                                                        |
| ---------------------------- |----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user views their card activity history by selecting an activity from a list, then explores related information such as data shared,<br>agreements or organizational details.                   |
| **Goal**                     | Reviewing usage and management events of the App.                                                                                                                                                  |
| **Preconditions**            | *None*                                                                                                                                                                                             |
| **Postconditions**           | *None*                                                                                                                                                                                             |
| **Triggered by**             | <ul><li>User selects 'Activities' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md).</li><li>User selects 'Activities' in [UC7.1 Show all available cards](UC7.1_ShowAllAvailableCards.md).</li></ul> |
| **Additional Documentation** | *None*                                                                                                                                                                                             |
| **Possible errors**          | *None*                                                                                                                                                                                             |
| **Logical test cases**       | <ul><li>[LTC30 View activity list](../logical-test-cases.md#ltc30)</li></ul>                                                                                                                       |

---

## Flow

| #       | Description                                                                                                             | Next                                                        |
|---------|-------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                    |                                                             |
| **1.1** | **System displays screen 'All activities'**<ul><li>Activities</li><li>Items: Activities</li><li>Actions: Back</li></ul> |                                                             |
| 1.1a    | User selects Activity                                                                                                   | Go to: [UC6.3 Show card details](UC6.3_ShowHistoryEvent.md) |
| 1.1b    | User selects Back                                                                                                       | Go to: [UC7.2 Show card details](UC7.2_ShowCardDetails.md)  |