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
| **Logical test cases**       | <ul><li>[LTC22 View complete history](../logical-test-cases.md#ltc22)</li></ul>                                                                                                                    |

---

## Flow

| #       | Description                                                                                                             | Next                                                         |
|---------|-------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                    |                                                              |
| **1.1** | **System displays screen 'All activities'**<ul><li>Activities</li><li>Items: Activities</li><li>Actions: Back</li></ul>*Displayed activities* — the system displays all usage and management activities sorted by date and grouped per calendar month:<ul><li>Card issuance ('Card created')</li><li>Card renewal ('Card renewed')</li><li>Successful data sharing ('Data shared')</li><li>Stopped data sharing ('Sharing stopped')</li><li>Failed data sharing where data was sent ('Sharing may have failed')</li><li>Failed data sharing where no data was sent ('Sharing failed')</li><li>Successful login ('Logged in')</li><li>Stopped login ('Log in is stopped')</li><li>Failed login where data was sent ('Login may have failed')</li><li>Failed login where no data was sent ('Login failed')</li><li>Card deletion ('Card deleted')</li> |                                                              |
| 1.1a    | User selects Activity                                                                                                   | Go to: [UC6.3 Show history event](UC6.3_ShowHistoryEvent.md) |
| 1.1b    | User selects Back                                                                                                       | Back                                                         |
