# Notifications

This page describes the different types of notifications a user can receive when using NL Wallet. For each notification type, this page provides:

- The trigger condition that causes the notification to be displayed.
- The schedule when the notification is shown (immediate or at a specific time).
- The message content for both OS notifications (push notifications) and in-app notifications.
- The action taken when the user interacts with the notification.

Notifications are categorized by card type (PID or EAA) and by the event that triggers them (expires soon, expired, revoked, or corrupted) and other types of notifications. 
The use case descriptions in the functional design specify how these notifications integrate into user workflows.

---

## PID (Personal Identity Document) Notifications

### PID Expires Soon

| Aspect                  | Description                                                                            |
|-------------------------|----------------------------------------------------------------------------------------|
| **Trigger**             | Card will expire in 14 days                                                            |
| **Schedule**            | 10:00 AM Dutch time                                                                    |
| **OS Notification**     | In # days, your app will not working. Solve this in the app.                           |
| **In-app Notification** | **In # days, your app will not be working**<br>Solve this in a few steps.              |
| **Action**              | <ul><li>Go to: [UC7.2 Show card details](use-cases/UC7.2_ShowCardDetails.md)</li></ul> |

### PID Expired

| Aspect                  | Description                                                                            |
|-------------------------|----------------------------------------------------------------------------------------|
| **Trigger**             | Card expiry date reached, user can not use the app                                     |
| **Schedule**            | Immediate                                                                              |
| **OS Notification**     | Attention: your NL Wallet does not work right now. Solve this in the app.              |
| **In-app Notification** | **Attention: your NL Wallet is not working right now**<br>Solve this in a few steps.   |
| **Action**              | <ul><li>Go to: [UC7.2 Show card details](use-cases/UC7.2_ShowCardDetails.md)</li></ul> |

### PID Revoked

| Aspect                  | Description                                                                            |
|-------------------------|----------------------------------------------------------------------------------------|
| **Trigger**             | Issuer revokes card, user can not use the app                                          |
| **Schedule**            | Immediate                                                                              |
| **OS Notification**     | Attention: your NL Wallet does not work right now. Solve this in the app.              |
| **In-app Notification** | **Attention: your NL Wallet is not working right now**<br>Solve this in a few steps.   |
| **Action**              | <ul><li>Go to: [UC7.2 Show card details](use-cases/UC7.2_ShowCardDetails.md)</li></ul> |

## EAA (Electronic Attestation of Attributes) Notifications

### EAA Expires Soon

| Aspect                  | Description                                                                            |
|-------------------------|----------------------------------------------------------------------------------------|
| **Trigger**             | Card will expire in 14 days                                                            |
| **Schedule**            | 10:00 AM Dutch time                                                                    |
| **OS Notification**     | {Card} expires in # days. Replace this card if you still need it.                      |
| **In-app Notification** | **{Card} expires in # days**<br>Replace this card if you still need it.                |
| **Action**              | <ul><li>Go to: [UC7.2 Show card details](use-cases/UC7.2_ShowCardDetails.md)</li></ul> |

### EAA Expired

| Aspect                  | Description                                                                            |
|-------------------------|----------------------------------------------------------------------------------------|
| **Trigger**             | Card expiry date reached, user can still use the app                                   |
| **Schedule**            | Immediate                                                                              |
| **OS Notification**     | {Card} expired. Replace this card if you still need it.                                |
| **In-app Notification** | **{Card} expired**<br>Replace this card if you still need it.                          |
| **Action**              | <ul><li>Go to: [UC7.2 Show card details](use-cases/UC7.2_ShowCardDetails.md)</li></ul> |

### EAA Revoked

| Aspect                  | Description                                                                            |
|-------------------------|----------------------------------------------------------------------------------------|
| **Trigger**             | Issuer revokes card, user can still use the app                                        |
| **Schedule**            | Immediate                                                                              |
| **OS Notification**     | {Card} withdrawn by issuer. Replace this card if you still need it.                    |
| **In-app Notification** | **{Card} withdrawn by issuer**<br>Replace this card if you still need it.              |
| **Action**              | <ul><li>Go to: [UC7.2 Show card details](use-cases/UC7.2_ShowCardDetails.md)</li></ul> |

## Other Notifications

### App tour

| Aspect                  | Description                                                           |
|-------------------------|-----------------------------------------------------------------------|
| **Trigger**             | First dashboard display                                               |
| **Schedule**            | Show on the user's first app dashboard visit                          |
| **OS Notification**     | *None*                                                                |
| **In-app Notification** | Discover everything in short videos                                   |
| **Action**              | <ul><li>Go to: [UC1.3 App tour](use-cases/UC1.3_AppTour.md)</li></ul> |

### Configure notifications

| Aspect                  | Description                                                                                                                                                                                                                 |
|-------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Trigger**             | Second dashboard display                                                                                                                                                                                                    |
| **Schedule**            | Show on the user's second app dashboard visit                                                                                                                                                                               |
| **OS Notification**     | *None*                                                                                                                                                                                                                      |
| **In-app Notification** | **Do you want to get important updates** <br> You will get message about your card changes, app updates, and other important messages. Even when you do not open the app.<br>You can turn this on or off later in Settings. |
| **Action(s)**           | <ul><li>Not now<br>&rarr; Notification dismissed</li><li>Yes, turn it on<br>&rarr; OS asks for permission</li></ul>                                                                                                         |
