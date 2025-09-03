# Use Case 9.6 Get help

## Overview

| Aspect                       | Description                                                                                 |
|------------------------------|---------------------------------------------------------------------------------------------|
| **Summary**                  | The user ask for help from within the app and is shown a message on how to retrieve help.   |
| **Goal**                     | Getting help regarding usage of the app.                                                    |
| **Preconditions**            | *None*                                                                                      |
| **Postconditions**           | *None*                                                                                      |
| **Triggered by**             | <ul><li>User selects 'Help' in UC1.1, UC1.2, UC2.6, UC3.1, UC4.1, UC5.1 or UC5.2.</li></ul> |
| **Additional documentation** | *None*                                                                                      |
| **Possible errors**          | *None*                                                                                      |
| **Logical test cases**       | <ul><li>[LTC42 Get help on PIN screen](../logical-test-cases.md#ltc42)</li></ul>            |


---

## Flow

| #       | Description                                                                                                                                                  | Next |
|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                         |      |
| **1.1** | **System displays screen 'Need help?'**<ul><li>Message: During the pilot you can reach out to someone of the NL Wallet-team.</li><li>Actions: Back</li></ul> |      |
| 1.1a    | User selects Back                                                                                                                                            | Back |
<style>td {vertical-align:top}</style>