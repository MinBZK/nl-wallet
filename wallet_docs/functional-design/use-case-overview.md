# Use case overview

The three flow diagrams present on this page display the relation between use cases. This relation is based
on which use cases can be triggered by a given use case, therefore giving a representation
on which flows a user can have between various use cases. For readability the relations
between the various use cases are divided into 3 diagrams, one showing the use case flow
for opening and onboarding the app, the second displays the use case flow from the app dashboard,
and the third showing the use case flow from viewing the app menu.

<br><br>
## Opening and onboarding use case flow

```{mermaid}
flowchart LR
INTRODUCE["<a href='use-cases/UC1.1_IntroduceTheApp.html'>1.1 Introduce the app</a>"]
OPEN["<a href='use-cases/UC1.2_OpenTheApp.html'>1.2 Open the app</a>"]

SETUP_PIN["<a href='use-cases/UC2.1_SetupRemotePinAndBiometricsUnlock.html'>2.1 Setup PIN & biometrics</a>"]
UNLOCK["<a href='use-cases/UC2.3_UnlockTheApp.html'>2.3 Unlock the app</a>"]
RECOVER_PIN["<a href='use-cases/UC2.3.2_RecoverPIN.html'>2.3.2 Recover PIN</a>"]

OBTAIN_PID["<a href='use-cases/UC3.1_ObtainPidFromProvider.html'>3.1 Obtain PID</a>"]
WIPE["<a href='use-cases/UC9.4_WipeAllAppData.html'>9.4 Wipe app data</a>"]

ALL_CARDS["<a href='use-cases/UC7.1_ShowAllAvailableCards.html'>7.1 Show all cards</a>"]
TRANSFER["<a href='use-cases/UC9.10_TransferWallet.html'>9.10 Transfer wallet</a>"]
END((" "))

APP_INFO["<a href='use-cases/UC9.2_GetAppInformation.html'>9.2 Get app info</a>"]

OPEN --> UNLOCK
OPEN --> INTRODUCE

UNLOCK --> ALL_CARDS
UNLOCK --> OBTAIN_PID
UNLOCK --> APP_INFO
UNLOCK --> RECOVER_PIN

INTRODUCE --> SETUP_PIN

OBTAIN_PID --> ALL_CARDS
OBTAIN_PID --> TRANSFER
OBTAIN_PID --> WIPE

WIPE --> INTRODUCE

TRANSFER --> END

RECOVER_PIN --> ALL_CARDS

SETUP_PIN --> OBTAIN_PID
SETUP_PIN --> APP_INFO

```
<br><br><br>
## Dashboard use case flow

The two circle nodes in the flow diagram underneath depict the same point in the flow. When user reaches the second circle node
the user will start again at the first circle node.

```{mermaid}
flowchart LR
START((" "))
END((" "))
APP_TOUR["<a href='use-cases/UC1.3_AppTour.html'>1.3 Watch explainer videos</a>"]

OBTAIN_CARDS["<a href='use-cases/UC4.1_ObtainCardsFromEAAIssuer.html'>4.1 Obtain EAA</a>"]

SHARE["<a href='use-cases/UC5.1_ShareDataWithRP.html'>5.1 Share data with RP</a>"]
LOGIN["<a href='use-cases/UC5.2_LoginToApplicationOfRP.html'>5.2 Log in to RP</a>"]

ALL_HISTORY["<a href='use-cases/UC6.1_ShowCompleteUsageAndManagementHistory.html'>6.1 Browse app activity</a>"]
CARD_HISTORY["<a href='use-cases/UC6.2_ShowUsageAndManagementHistoryOfCard.html'>6.2 Browse card activity</a>"]
HISTORY_EVENT["<a href='use-cases/UC6.3_ShowHistoryEvent.html'>6.3 View activity details</a>"]

ALL_CARDS["<a href='use-cases/UC7.1_ShowAllAvailableCards.html'>7.1 Show all cards</a>"]
CARD["<a href='use-cases/UC7.2_ShowCardDetails.html'>7.2 View card Details</a>"]

MENU["<a href='use-cases/UC9.1_ShowAppMenu.html'>9.1 View app menu</a>"]
APP_INFO["<a href='use-cases/UC9.2_GetAppInformation.html'>9.2 Get app info</a>"]
QR["<a href='use-cases/UC9.9_ScanQR.html'>9.9 Scan QR</a>"]

RECOVER_PIN["<a href='use-cases/UC2.3.2_RecoverPIN.html'>2.3.2 Recover PIN</a>"]
RENEW_PID["<a href='use-cases/UC3.2_RenewPid.html'>3.3 Renew PID</a>"]

START --> ALL_CARDS

ALL_CARDS --> APP_TOUR
ALL_CARDS --> MENU
ALL_CARDS --> QR
ALL_CARDS --> ALL_HISTORY
ALL_CARDS --> CARD
ALL_CARDS --> APP_INFO

OBTAIN_CARDS --> END
OBTAIN_CARDS --> RECOVER_PIN

SHARE --> END
SHARE --> RECOVER_PIN

LOGIN --> END
LOGIN --> RECOVER_PIN

RECOVER_PIN --> END

QR --> OBTAIN_CARDS
QR --> SHARE
QR --> LOGIN

ALL_HISTORY --> HISTORY_EVENT

CARD_HISTORY --> HISTORY_EVENT

CARD --> CARD_HISTORY
CARD --> RENEW_PID

RENEW_PID --> END
RENEW_PID --> RECOVER_PIN

```

<br><br><br>
## Menu use case flow

```{mermaid}
flowchart LR
ELLIP1[⋯]
ELLIP2[⋯]
ELLIP3[⋯]
INTRODUCE["<a href='use-cases/UC1.1_IntroduceTheApp.html'>1.1 Introduce the app</a>"]
APP_TOUR["<a href='use-cases/UC1.3_AppTour.html'>1.3 Watch explainer videos</a>"]

TOGGLEBIO["<a href='use-cases/UC2.2_ChangeBiometricUnlock.html'>2.2 Toggle biometrics</a>"]
UNLOCK["<a href='use-cases/UC2.3_UnlockTheApp.html'>2.3 Unlock the app</a>"]
CHANGEPIN["<a href='use-cases/UC2.6_ChangeRemotePIN.html'>2.6 Change PIN</a>"]

OBTAIN_CARDS["<a href='use-cases/UC4.1_ObtainCardsFromEAAIssuer.html'>4.1 Obtain EAA</a>"]

SHARE["<a href='use-cases/UC5.1_ShareDataWithRP.html'>5.1 Share data with RP</a>"]
LOGIN["<a href='use-cases/UC5.2_LoginToApplicationOfRP.html'>5.2 Log in to RP</a>"]

ALL_HISTORY["<a href='use-cases/UC6.1_ShowCompleteUsageAndManagementHistory.html'>6.1 Browse app activity</a>"]
HISTORY_EVENT["<a href='use-cases/UC6.3_ShowHistoryEvent.html'>6.3 View activity details</a>"]

MENU["<a href='use-cases/UC9.1_ShowAppMenu.html'>9.1 View app menu</a>"]
APP_INFO["<a href='use-cases/UC9.2_GetAppInformation.html'>9.2 Get app info</a>"]
CHANGELANG["<a href='use-cases/UC9.3_ChangeAppLanguage.html'>9.3 Change app language</a>"]
WIPE["<a href='use-cases/UC9.4_WipeAllAppData.html'>9.4 Wipe app data</a>"]
HELP["<a href='use-cases/UC9.6_GetHelp.html'>9.6 Get help</a>"]
LOGOUT["<a href='use-cases/UC9.7_LogoutOffTheApp.html'>9.7 Logout of app</a>"]
QR["<a href='use-cases/UC9.9_ScanQR.html'>9.9 Scan QR</a>"]

WIPE --> INTRODUCE

OBTAIN_CARDS --> ELLIP1

SHARE --> ELLIP2

LOGIN --> ELLIP3

QR --> OBTAIN_CARDS
QR --> SHARE
QR --> LOGIN

MENU --> APP_TOUR
MENU --> QR
MENU --> ALL_HISTORY
MENU --> APP_INFO
MENU --> LOGOUT
MENU --> HELP

MENU --> CHANGEPIN
MENU --> TOGGLEBIO
MENU --> CHANGELANG
MENU --> WIPE

ALL_HISTORY --> HISTORY_EVENT

LOGOUT --> UNLOCK

classDef trans fill:transparent,stroke:transparent
class ELLIP1 trans
class ELLIP2 trans
class ELLIP3 trans
```