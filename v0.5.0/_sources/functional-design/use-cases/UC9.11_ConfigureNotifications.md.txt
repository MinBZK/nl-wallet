# Use Case 9.11 Configure Notifications

## Overview

| Aspect                       | Description                                                                                                 |
|------------------------------|-------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user opens the notification settings screen from the menu, where they can turn notifications on or off. |
| **Goal**                     | Turning notifications on or off.                                                                            |
| **Preconditions**            | *None*                                                                                                      |
| **Postconditions**           | Notifications are turned on or off.                                                                         |
| **Triggered by**             | <ul><li>User selects 'Notifications' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md).</li></ul>              |
| **Additional Documentation** | *None*                                                                                                      |
| **Possible errors**          | *None*                                                                                                      |
| **Logical test cases**       | <ul><li>[LTC72 Configure notifications](../logical-test-cases.md#ltc72)</li></ul>                           |

---

## Flow

| #       | Description                                                                                                                                              | Next |
|---------|----------------------------------------------------------------------------------------------------------------------------------------------------------|------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                     |      |
| **1.1** | **System displays screen 'Notifications'**<ul><li>Toggle: Push notification</li><li>Current language is selected</li><li>Actions: Back</li></ul>         |      |
| 1.1a    | User toggles notifications on<br>&rarr; Notifications are enabled (When user has not given permission, OS ask for permission to send push notifications) |      |
| 1.1b    | User toggles notifications off<br>&rarr; Notifications are disabled                                                                                      |      |
| 1.1c    | User selects Back                                                                                                                                        | Back |
