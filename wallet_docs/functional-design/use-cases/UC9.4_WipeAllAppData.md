# Use Case 9.4 Wipe all app data

## Overview

| Aspect                       | Description                                                                                                                                                                         |
|------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user chooses to delete all app data by confirming the deletion message, which deletes app data and restarts the onboarding process.                                             |
| **Goal**                     | Wiping all data from the app.                                                                                                                                                       |
| **Preconditions**            | *None*                                                                                                                                                                              |
| **Postconditions**           | <ul><li>App data is deleted.</li></ul>                                                                                                                                              |
| **Triggered by**             | <ul><li>User selects 'Remove data' in [UC9.1 Show app menu](UC9.1_ShowAppMenu.md)</li><li>User selects 'Forgot PIN' in UC2.2, UC2.3, UC2.6, UC3.1, UC4.1, UC5.1 or UC5.2.</li></ul> |
| **Additional Documentation** | *None*                                                                                                                                                                              |
| **Possible errors**          | *None*                                                                                                                                                                              |
| **Logical test cases**       | <ul><li>[LTC28 Delete App data](../logical-test-cases.md#ltc28)</li><li>[LTC29 Cancel app data deletion](../logical-test-cases.md#ltc29)</li></ul>                                  |

---

## Flow

| #       | Description                                                                                                                              | Next                                                       |
|---------|------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                     |                                                            |
| **1.1** | **System displays screen 'Wipe app data'**<ul><li>Message: Do you want to delete all data?</li><li>Actions: Yes Delete, Cancel</li></ul> |                                                            |
| 1.1a    | User selects Yes Delete <br>&rarr; App deletes local data                                                                                | Go to: [UC1.1 Introduce the app](UC1.1_IntroduceTheApp.md) |
| 1.1b    | User selects Cancel                                                                                                                      | Back                                                       |
<style>td {vertical-align:top}</style>