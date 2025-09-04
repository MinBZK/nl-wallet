# Use Case 6.2 Show card history

## Overview

| Aspect                       | Description                                                                                                                                              |
|------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user reviews activity history specific to a selected card by choosing an activity, then explores related data, agreements or organizational details. |
| **Goal**                     | Reviewing usage and management events of a card                                                                                                          |
| **Preconditions**            | *None*                                                                                                                                                   |
| **Postconditions**           | *None*                                                                                                                                                   |
| **Triggered by**             | <ul><li>User selects 'Activities' in [UC7.2 Show Card Details](UC7.2_ShowCardDetails.md)</ul></li>                                                       |
| **Additional Documentation** | *None*                                                                                                                                                   |
| **Possible errors**          | *None*                                                                                                                                                   |
| **Logical test cases**       | <ul><li>[LTC31 View card-specific activity list](../logical-test-cases.md#ltc31)</li></ul>                                                               |

---

## Flow

| #       | Description                                                                                                                   | Next                                                        |
|---------|-------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                          |                                                             |
| **1.1** | **System displays screen 'Card activities'**<ul><li>Card activities</li><li>Items: Activities</li><li>Actions: Back</li></ul> |                                                             |
| 1.1a    | User selects Activity                                                                                                         | Go to: [UC6.3 Show card details](UC6.3_ShowHistoryEvent.md) |
| 1.1b    | User selects Back                                                                                                             | Go to: [UC7.2 Show card details](UC7.2_ShowCardDetails.md)  |