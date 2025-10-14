# Use Case 9.6 Get help

## Overview

| Aspect                       | Description                                                                               |
|------------------------------|-------------------------------------------------------------------------------------------|
| **Summary**                  | The user ask for help from within the app and is shown a message on how to retrieve help. |
| **Goal**                     | Getting help regarding usage of the app.                                                  |
| **Preconditions**            | *None*                                                                                    |
| **Postconditions**           | *None*                                                                                    |
| **Triggered by**             | <ul><li>User selects 'Need help?' in UC9.1</li></ul>                                      |
| **Additional documentation** | *None*                                                                                    |
| **Possible errors**          | *None*                                                                                    |
| **Logical test cases**       | <ul><li>[LTC42 Get help](../logical-test-cases.md#ltc42)</li></ul>                        |

---

## Flow

| #       | Description                                                                                                                                                   | Next                                  |
|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                          |                                       |
| **1.1** | **System displays screen 'Need help?'**<ul><li>Items: Cards, QR code, Share data, Activities, Security, App settings, Contact</li><li>Actions: Back</li></ul> |                                       |
| 1.1a    | User selects Cards                                                                                                                                            | Show placeholder 'under construction' |
| 1.1b    | User selects QR code                                                                                                                                          | Show placeholder 'under construction' |
| 1.1c    | User selects Share data                                                                                                                                       | Show placeholder 'under construction' |
| 1.1d    | User selects Activities                                                                                                                                       | Show placeholder 'under construction' |
| 1.1e    | User selects Security                                                                                                                                         | Show placeholder 'under construction' |
| 1.1f    | User selects App settings                                                                                                                                     | Show placeholder 'under construction' |
| 1.1g    | User selects Contact                                                                                                                                          | 1.2                                   |
| 1.1h    | User selects Back                                                                                                                                             | Back                                  |
| **1.2** | **System displays screen 'Contact?'**<ul><li>Message: Check website or call us to get to know more about the NL Wallet</li><li>Actions: Call, Back</li></ul>  |                                       |
| 1.2a    | User selects Call                                                                                                                                             | Show placeholder 'under construction' |
| 1.2b    | User selects Back                                                                                                                                             | Back                                  |
