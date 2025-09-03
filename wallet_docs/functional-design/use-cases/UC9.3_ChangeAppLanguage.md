# Use Case 9.3 Change app language

## Overview

| Aspect                       | Description                                                                                                |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------- |
| **Summary**                  | The user opens the language settings screen from the menu, where they can switch between Dutch and English |
| **Goal**                     | Changing the language of the app to either Dutch or English.                                               |
| **Preconditions**            | *None*                                                                                                     |
| **Postconditions**           | The Language of the app changed to the user selection.                                                     |
| **Triggered by**             | <ul><li>User selects 'Change Language' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md).</li></ul>           |
| **Additional Documentation** | *None*                                                                                                     |
| **Possible errors**          | *None*                                                                                                     |
| **Logical test cases**       | <ul><li>[LTC39 Select a new language](../logical-test-cases.md#ltc39)</li></ul>                            |

---

## Flow

| #       | Description                                                                                                                                      | Next |
|---------|--------------------------------------------------------------------------------------------------------------------------------------------------|------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                             |      |
| **1.1** | **System displays screen 'Change Language'**<ul><li>Options: Dutch, English</li><li>Current language is selected</li><li>Actions: Back</li></ul> |      |
| 1.1a    | User selects inactive option <br>&rarr; App language is changed                                                                                  |      |
| 1.1b    | User selects Back                                                                                                                                | Back |
<style>td {vertical-align:top}</style>