# Use Case 9.10 Transfer Wallet to another device

## Overview

| Aspect                       | Description                                                                                                                                                                                                                                                                |
|------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Summary**                  | When a user activates a second wallet, they can decide to move their original wallet to this new device. With the source device, they scan a QR displayed on the target device to initiate the transfer. Afterwards, the source device is empty.                           |
| **Goal**                     | Allow the user to transfer attestation data, usage log and user settings from one device to another.                                                                                                                                                                       |
| **Preconditions**            | <ul><li>Destination Wallet is a fresh installation of NL Wallet, that is just activated using DigiD, containing a valid PID.</li><li>Source Wallet is an existing, valid NL Wallet installation, containing attestations, transaction history and user settings.</li></ul> |
| **Postconditions**           | <ul><li>Destination Wallet is ‘active’, containing data from the Source Wallet</li><li>Source Wallet is ‘inactive’ and no longer usable</li></ul>                                                                                                                          |
| **Triggered by**             | <ul><li>User selects 'Yes move' in [UC3.1 Obtain PID](UC3.1_ObtainPidFromProvider.md)</li></ul>                                                                                                                                                                            |
| **Additional documentation** | <ul><li>[Wallet device transfer](../../architecture/use-cases/recovery_usecases.md)</li></ul>                                                                                                                                                                              |
| **Possible errors**          | <ul><li>[No Internet](../errors.md#no-internet)</li><li>[Server Unreachable](../errors.md#server-unreachable)</li></ul>                                                                                                                                                    |
| **Logical test cases**       | <ul><li>[LTC62 Transfer Wallet](../logical-test-cases.md#ltc62)</li><li>[LTC63 Stop transfer on source device](../logical-test-cases.md#ltc63)</li><li>[LTC64 Stop transfer on destination device](../logical-test-cases.md#ltc64)</li></ul>                               |

---

<br>

## Flow Overview

The following diagram indicates how the user transfers their wallet from one
device to another. They need to operate both devices.

```{mermaid}
sequenceDiagram
    autonumber

    actor User
    participant WS as Source Wallet
    participant WT as Destination Wallet

    User ->> WT: Start transfer
    WT ->> User: Show QR-code
    User ->> WS: Scan QR-code
    WT ->> WS: QR-code (with encryption key)
    User ->> WS: Confirm transfer with PIN
    WS ->> WT: Send encrypted wallet data (via backend)
    WT ->> User: Transfer complete
    WS ->> User: Transfer complete (Source wallet now deactivated)
```

---

<br>

## Flow Wallet Device Transfer (Source)

| #       | Description                                                                                                                                                        | Next                                                                    |
| ------- |--------------------------------------------------------------------------------------------------------------------------------------------------------------------| ----------------------------------------------------------------------- |
| **1**   | **PRIMARY SCENARIO**                                                                                                                                               |                                                                         |
| **1.1** | **User executes [UC9.9 Scan QR](UC9.9_ScanQR.md) to scan _wallet transfer QR_ from target device**                                                                 | 1.2                                                                     |
| **1.2** | **System displays screen 'Confirm transfer'**<ul><li>Message: Do you want to move your wallet?</li><li>Actions: Yes move, Stop</li></ul>                           |                                                                         |
| 1.2a    | User selects Yes move                                                                                                                                              | 1.3                                                                     |
| 1.2b    | User selects Stop                                                                                                                                                  | 2                                                                       |
| 1.2c    | Event: User stops on destination device                                                                                                                            | 3                                                                       |
| 1.2d    | Error: No internet                                                                                                                                                 | Error flow: [No Internet](../errors.md#no-internet)                     |
| **1.3** | **System executes partial flow [PF2.4 Confirm a protected action](../partial-flows/PF2.4_ConfirmProtectedAction.md)**<ul><li>Cancelable</li></ul>                  |                                                                         |
| 1.3a    | Result: Confirm                                                                                                                                                    | 1.4                                                                     |
| 1.3b    | Result: Cancel                                                                                                                                                     | 2                                                                       |
| 1.3c    | Result: Back                                                                                                                                                       | Back                                                                    |
| 1.3d    | Event: User stops on destination device                                                                                                                            | 3                                                                       |
| **1.4** | **System displays screen 'Transferring wallet'**<ul><li>Message: Wallet is being moved</li><li>Actions: Stop</li></ul>                                             |                                                                         |
| 1.4a    | Event: Transfer completed successfully                                                                                                                             | 1.5                                                                     |
| 1.4b    | Event: Transfer failed                                                                                                                                             | 4                                                                       |
| 1.4c    | Event: User stops on destination device                                                                                                                            | 3                                                                       |
| 1.4d    | User selects Stop                                                                                                                                                  | 2                                                                       |
| 1.4e    | Error: No internet                                                                                                                                                 | Error flow: [No Internet](../errors.md#no-internet)                     |
| 1.4f    | Error: Server unreachable                                                                                                                                          | Error flow: [Server Unreachable](../errors.md#server-unreachable)       |
| **1.5** | **System displays screen 'Move completed'**<ul><li>Message: Wallet successfully moved, this app is now empty</li><li>Actions: Create new NL wallet, Help</li></ul> |                                                                         |
| 1.5a    | User selects Create new NL wallet                                                                                                                                  | Go to: [UC1.1 Introduce the app](UC1.1_IntroduceTheApp.md)              |
| 1.5b    | User selects Help                                                                                                                                                  | Show placeholder 'under construction'                                   |
| **2**   | **STOP TRANSFER**                                                                                                                                                  |                                                                         |
| **2.1** | **System displays bottom sheet 'Are you sure you want to stop?'**<ul><li>Actions: Yes stop, No</li></ul>                                                           |                                                                         |
| 2.1a    | User selects Yes stop                                                                                                                                              | 2.2                                                                     |
| 2.1b    | User selects No                                                                                                                                                    | Back                                                                    |
| **2.2** | **System displays screen 'Stopped'**<ul><li>Actions: Close, Help</li></ul>                                                                                         |                                                                         |
| 2.2a    | User selects Close                                                                                                                                                 | Go to: [UC7.1 Show all available cards](UC7.1_ShowAllAvailableCards.md) |
| 2.2b    | User selects Help                                                                                                                                                  | Show placeholder 'under construction'                                   |
| **3**   | **WHEN DESTINATION DEVICE STOPS TRANSFER**                                                                                                                         |                                                                         |
| **3.1** | **System displays screen 'Stopped on destination device'**<ul><li>Actions: Close, Help</li></ul>                                                                   |                                                                         |
| 3.1a    | User selects Close                                                                                                                                                 | Go to: [UC7.1 Show all available cards](UC7.1_ShowAllAvailableCards.md) |
| 3.1b    | User selects Help                                                                                                                                                  | Show placeholder 'under construction'                                   |
| **4**   | **WHEN WALLET TRANSFER FAILS**                                                                                                                                     |                                                                         |
| **4.1** | **System displays screen 'Move failed'**<ul><li>Message: Wallet could not be moved</li><li>Actions: See details, Close</li></ul>                                   |                                                                         |
| 4.1a    | User selects See details                                                                                                                                           | 4.2                                                                     |
| 4.1b    | User selects Close                                                                                                                                                 | End, Go to: [UC3.1 Obtain PID](UC3.1_ObtainPidFromProvider.md) 1.8      |
| **4.2** | **System displays bottom sheet 'Error details'**<ul><li>Actions: Close</li></ul>                                                                                   |                                                                         |
| 4.2a    | User selects Close                                                                                                                                                 | 4.1                                                                     |

---

<br>

## Flow Wallet Device Transfer (Destination)

| #       | Description                                                                                                                                                                    | Next                                                                    |
|---------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------| ----------------------------------------------------------------------- |
| **1**   | **PRIMARY SCENARIO**                                                                                                                                                           |                                                                         |
| **1.1** | **System displays screen 'Scan QR'**<ul><li>Message: Use the QR scanner on your other device to start transfer</li><li>Actions: Center QR code on screen, Help, Back</li></ul> |                                                                         |
| 1.1a    | Event: Source device contacts target device, upon scanning QR.                                                                                                                 | 1.2                                                                     |
| 1.1b    | User selects Center QR code on screen <br>&rarr; App displays modal with centered QR                                                                                           |                                                                         |
| 1.1c    | User selects Help                                                                                                                                                              | Show placeholder 'under construction'                                   |
| 1.1d    | User selects Back                                                                                                                                                              | Back                                                                    |
| 1.1e    | Error: No internet                                                                                                                                                             | Error flow: [No Internet](../errors.md#no-internet)                     |
| **1.2** | **System displays screen 'Confirm on your other device'**<ul><li>Message: Confirm and enter PIN on other device</li><li>Actions: Help, Stop</li></ul>                          |                                                                         |
| 1.2a    | Event: Source device confirms initiation of wallet transfer, upon user approval.                                                                                               | 1.3                                                                     |
| 1.2b    | Event: User stops on source device                                                                                                                                             | 3                                                                       |
| 1.2c    | User selects Stop                                                                                                                                                              | 2                                                                       |
| 1.2e    | User selects Help                                                                                                                                                              | Show placeholder 'under construction'                                   |
| 1.2h    | Error: No internet                                                                                                                                                             | Error flow: [No Internet](../errors.md#no-internet)                     |
| 1.2i    | Error: Server unreachable                                                                                                                                                      | Error flow: [Server Unreachable](../errors.md#server-unreachable)       |
| **1.3** | **System displays screen 'Transferring wallet'**<ul><li>Message: Wallet is being moved to this device</li><li>Actions: Stop</li></ul>                                          |                                                                         |
| 1.3a    | Event: Wallet transfer completed successfully                                                                                                                                  | 1.4                                                                     |
| 1.3b    | Event: Wallet transfer failed                                                                                                                                                  | 4                                                                       |
| 1.3c    | Event: User stops on source device                                                                                                                                             | 3                                                                       |
| 1.3d    | User selects Stop                                                                                                                                                              | 2                                                                       |
| 1.3e    | Error: No internet                                                                                                                                                             | Error flow: [No Internet](../errors.md#no-internet)                     |
| 1.3f    | Error: Server unreachable                                                                                                                                                      | Error flow: [Server Unreachable](../errors.md#server-unreachable)       |
| **1.4** | **System displays screen 'Success'**<ul><li>Message: Your wallet is ready</li><li>Actions: To my overview, Help</li></ul>                                                      |                                                                         |
| 1.4a    | User selects To my overview                                                                                                                                                    | Go to: [UC7.1 Show all available cards](UC7.1_ShowAllAvailableCards.md) |
| 1.4b    | User selects Help                                                                                                                                                              | Show placeholder 'under construction'                                   |
| **2**   | **STOP TRANSFERRING**                                                                                                                                                          |                                                                         |
| **2.1** | **System displays bottom sheet 'Are you sure you want to stop?'**<ul><li>Actions: Yes stop, No</li></ul>                                                                       |                                                                         |
| 2.1a    | User selects Yes stop                                                                                                                                                          | 2.2                                                                     |
| 2.1b    | User selects No                                                                                                                                                                | Back                                                                    |
| 2.1c    | Error: No internet                                                                                                                                                             | Error flow: [No Internet](../errors.md#no-internet)                     |
| **2.2** | **System displays screen 'Stopped'**<ul><li>Actions: Close, Help</li></ul>                                                                                                     |                                                                         |
| 2.2a    | User selects Close                                                                                                                                                             | End, Go to: [UC3.1 Obtain PID](UC3.1_ObtainPidFromProvider.md) 1.8      |
| 2.2b    | User selects Help                                                                                                                                                              | Show placeholder 'under construction'                                   |
| **3**   | **WHEN USER STOPS ON SOURCE DEVICE**                                                                                                                                           |                                                                         |
| **3.1** | **System displays screen 'Stopped from source device'**<ul><li>Actions: Try again, Help</li></ul>                                                                              |                                                                         |
| 3.1a    | User selects Try again                                                                                                                                                         | End, Go to: [UC3.1 Obtain PID](UC3.1_ObtainPidFromProvider.md) 1.8      |
| 3.1b    | User selects Help                                                                                                                                                              | Show placeholder 'under construction'                                   |
| **4**   | **WHEN WALLET TRANSFER FAILS**                                                                                                                                                 |                                                                         |
| **4.1** | **System displays screen 'Move failed'**<ul><li>Message: Wallet could not be moved</li><li>Actions: See details, Try again</li></ul>                                           |                                                                         |
| 4.1a    | User selects See details                                                                                                                                                       | 4.2                                                                     |
| 4.1b    | User selects Try again                                                                                                                                                         | End, Go to: [UC3.1 Obtain PID](UC3.1_ObtainPidFromProvider.md) 1.8      |
| **4.2** | **System displays bottom sheet 'Error details'**<ul><li>Actions: Close</li></ul>                                                                                               |                                                                         |
| 4.2a    | User selects Close                                                                                                                                                             | 4.1                                                                     |
