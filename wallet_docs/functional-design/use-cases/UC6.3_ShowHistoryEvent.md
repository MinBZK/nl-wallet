# Use Case 6.2 Show card history

## Overview

| Aspect                       | Description                                                                                                                                                                                                                         |
|------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | The user reviews activity history specific to a selected event, then explores related data, agreements or organizational details.                                                                                                   |
| **Goal**                     | Reviewing usage events                                                                                                                                                                                                              |
| **Preconditions**            | *None*                                                                                                                                                                                                                              |
| **Postconditions**           | *None*                                                                                                                                                                                                                              |
| **Triggered by**             | <ul><li>User selects an event in [UC6.1 Show complete history](UC6.1_ShowCompleteUsageAndManagementHistory.md).</li><li>User selects an event in [UC6.2 Show card history](UC6.2_ShowUsageAndManagementHistoryOfCard.md).</li></ul> |
| **Additional Documentation** | *None*                                                                                                                                                                                                                              |
| **Possible errors**          | *None*                                                                                                                                                                                                                              |
| **Logical test cases**       | <ul><li>[LTC31 View card-specific activity list](../logical-test-cases.md#ltc31)</li><li>[LTC30 View activity list](../logical-test-cases.md#ltc30)</li></ul>                                                                    |
 
---

## Flow

| #       | Description                                                                                                                                                  | Next                                      |
|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------|
| **1**   | **PRIMARY SCENARIO**                                                                                                                                         |                                           |
| **1.1** | **System displays screen 'Activity'**<ul><li>[Activity]</li><li>Actions: View data, Read the agreement, About [organization], Report Problem, Back</li></ul> |                                           |
| 1.1a    | User selects View data                                                                                                                                       | 2                                         |
| 1.1b    | User selects Read the agreement                                                                                                                              | 5                                         |
| 1.1c    | User selects About [organization]                                                                                                                            | 6                                         |
| 1.1d    | User selects Report Problem                                                                                                                                  | 4                                         |
| 1.1e    | User selects Back                                                                                                                                            | Back                                      |
| **2**   | **INSPECT ACTIVITY DETAILS**                                                                                                                                 |                                           |
| **2.1** | **System displays screen 'Data activity'**<ul><li># from [Card Title]</li><li>Actions: Details Incorrect, Help, Back</li></ul>                               |                                           |
| 2.1a    | User selects Details Incorrect                                                                                                                               | 3                                         |
| 2.1b    | User selects Help                                                                                                                                            | Go to: [UC9.6 Get help](UC9.6_GetHelp.md) |
| 2.1c    | User selects Back                                                                                                                                            | Back                                      |
| **3**   | **WHEN CARD DETAILS ARE INCORRECT**                                                                                                                          |                                           |
| **3.1** | **System displays screen 'Details incorrect'**<ul><li>Message: Are your details incorrect?</li><li>Actions: Back</li></ul>                                   |                                           |
| 3.1a    | User selects Back                                                                                                                                            | Back                                      |
| **4**   | **REPORT PROBLEM**                                                                                                                                           |                                           |
| **4.1** | **System displays screen 'Under construction'**<ul><li>Actions: Back</li></ul>                                                                               |                                           |
| 4.1a    | User selects Back                                                                                                                                            | Back                                      |
| **5.1** | **System displays screen 'Agreements'**<ul><li>Message: The following agreements apply</li><li>Actions: Privacy policy, Back</li></ul>                       |                                           |
| 5.1a    | User selects Privacy policy <br>&rarr; System opens browser and suspends app                                                                                 |                                           |
| 5.1b    | User selects Back                                                                                                                                            | Back                                      |
| **6**   | **INSPECT ORGANIZATION DETAILS**                                                                                                                             |                                           |
| **6.1** | **System displays screen 'About organization'**<ul><li>About [Organization]</li><li>Actions: Report Problem, Help, Back</li></ul>                            |                                           |
| 6.1a    | User selects Report Problem                                                                                                                                  | 4                                         |
| 6.1b    | User selects Help                                                                                                                                            | Go to: [UC9.6 Get help](UC9.6_GetHelp.md) |
| 6.1c    | User selects Back                                                                                                                                            | 1.2                                       |
<style>td {vertical-align:top}</style>