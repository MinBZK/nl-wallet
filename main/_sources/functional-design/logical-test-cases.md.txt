# Logical Test Cases

## LTCs

This page describes the logical test cases (LTCs) that are used to verify the correct functionality of the wallet app.
Test are written in Gherkin syntax with a Given When Then structure. Each LTC is implemented with at least one manual
or automated tests.

### LTC1

#### PID issuance

**Given** user has completed security setup<br>
**When** user authenticates at auth server<br>
**Then** system displays issued attributes to user for verification<br>
**When** user adds attributes<br>
**And** confirms using their PIN<br>
**Then** system displays message that wallet is created<br>
**And** provides a link to the dashboard<br>

---

### LTC2

#### Issuance fails

**Given** user has completed security setup<br>
**And** user authenticates at auth server<br>
**When** issuance fails<br>
**Then** system displays message that issuance failed<br>
**And** provides a link to try again<br>

---

### LTC3

#### Authentication at auth server fails

**Given** user has completed security setup<br>
**And** user authentication at auth server fails<br>
**Then** system displays message that authentication failed<br>
**And** provides a link to try again<br>

---

### LTC4

#### Rejects issued attributes

**Given** user has completed security setup<br>
**And** user authenticates at auth server<br>
**And** system displays issued attributes to user for verification<br>
**When** user rejects issued attributes<br>
**Then** provides a link to try again<br>

---

### LTC5

#### Disclosure based Issuance

**Given** user has completed PID setup and opened the app<br>
**When** user invokes a universal link from a (Q)EAA issuer<br>
**Then** system requests approval for disclosure<br>
**When** user approves with PIN<br>
**Then** system validates PIN and proceeds<br>
**And** system requests approval for attestations<br>
**When** user approves with PIN<br>
**Then** system issues card and displays it to user<br>
**And** provides a link to the dashboard<br>


---

### LTC6

#### Invalid universal link

**Given** user invokes an invalid universal link<br>
**Then** system informs user link is not recognized<br>

---

<!-- Manual  -->

### LTC7

#### Cross-device generic issuance

**Given** user is on a device with wallet installed<br>
**When** relying party presents a QR code<br>
**And** user scans it with wallet<br>
**Then** system requests approval for disclosure<br>
**When** user approves with PIN<br>
**Then** system validates PIN and proceeds<br>
**And** system requests approval for attestations<br>
**When** user approves with PIN<br>
**Then** system issues card and displays it to user<br>
**And** provides a link to the dashboard<br>

---

### LTC8

#### Reject disclosure of attributes

**Given** user starts card issuance<br>
**When** system asks for disclosure consent<br>
**And** user selects 'stop'<br>
**Then** system confirms cancellation<br>

---

### LTC9

#### No cards to be issued

**Given** user has no cards to be issued available at EAA issuer<br>
**When** user performs disclosure based issuance to retrieve cards<br>
**Then** system displays error message that no cards are available for issuance<br>

---

### LTC10

#### Wallet does not contain requested attributes

**Given** wallet does not contain attributes to fulfill a disclosure request
from an issuer<br>
**When** user invokes a universal link from a (Q)EAA issuer<br>
**Then** System displays an error message with instructions<br>
**When** user selects 'see details'<br>
**Then** system displays a bottom sheet with app information<br>

---

### LTC11

#### Renew card

**Given** user has an EAA card in its wallet<br>
**When** user invokes a universal link from issuer of card<br>
**Then** system requests approval for disclosure<br>
**When** user approves with PIN<br>
**Then** system validates PIN and proceeds<br>
**And** system requests approval card renewal<br>
**When** user approves with PIN<br>
**Then** system renews card<br>
**And** provides a link to the dashboard<br>
**And** merges history of old and new cards<br>
**And** adds a card renewel event to the history<br>

---

### LTC12

#### View introduction

**Given** the app is opened<br>
**And** user has not completed introduction<br>
**When** user navigates through the introduction screens<br>
**Then** system displays set pin screen<br>

---

### LTC13

#### Skip introduction

**Given** the app is opened<br>
**And** user has not completed introduction<br>
**When** user skips the introduction screens<br>
**Then** systems displays the privacy introduction screen<br>

---

### LTC14

#### View app tour

**Given** user has not closed app after obtaining PID<br>
**When** user views the app dashboard<br>
**Then** system displays a non-dismissible banner<br>
**When** user views the menu<br>
**Then** system displays an app tour menu item<br>
**When** user selects app tour<br>
**Then** system displays an app tour overview with a list of video items<br>
**When** user selects a video<br>
**Then** system opens the videoplayer<br>
**And** videoplayer contains correct controls<br>

---

### LTC15

#### Share data

**Given** user has completed PID setup and opened the app<br>
**When** user starts disclosure process at relying party<br>
**Then** system requests user consent<br>
**When** user approves with PIN<br>
**Then** system discloses attributes to relying party<br>
**And** system displays data shared message<br>

---

<!-- Manual  -->

### LTC16

#### Cross-device share data

**Given** user starts disclosure on a non-mobile device<br>
**When** user scans QR code with wallet<br>
**Then** system validates relying party URL<br>
**When** user confirms to proceed<br>
**And** approves disclosure with PIN<br>
**Then** system completes disclosure<br>
**And** displays a success message<br>

---

### LTC17

#### Decline consent to share data

**Given** user is shown consent screen<br>
**When** user selects 'Stop'<br>
**Then** system confirms cancellation<br>
**When** user confirms<br>
**Then** system displays the stopped screen<br>
**And** provides a link to the dashboard<br>

---

### LTC18

#### RP Login

**Given** user has completed PID setup and opened the app<br>
**When** user starts login process at relying party<br>
**Then** system requests consent to disclose BSN<br>
**When** user approves with PIN<br>
**Then** system discloses BSN to relying party<br>
**And** relying pary displays login success message<br>

---

<!-- Manual  -->

### LTC19

#### Cross-device login

**Given** user starts login on a non-mobile device<br>
**When** user scans QR code with wallet<br>
**Then** system validates relying party URL<br>
**When** user confirms to continue<br>
**And** approves disclosure with PIN<br>
**Then** login is completed<br>
**And** system displays success message<br>

---

### LTC20

#### Disclosure fails

**Given** user has completed security setup<br>
**When** user starts a disclosure flow<br>
**And** disclosure fails<br>
**And** system displays message disclosure failed<br>
**And** provides a link to try again<br>

---

### LTC21

#### Delete App data

**Given** user has completed PID setup and opened the app<br>
**When** user selects 'Remove data' from the settings menu<br>
**Then** system displays a prompt with 'Cancel' and 'Yes, Delete' options<br>
**When** user selects 'Yes, Delete'<br>
**Then** system displays the introduction screen<br>

---

### LTC22

#### View activity list

**Given** user has completed PID setup and opened the app<br>
**When** user selects 'activities' from settings or dashboard<br>
**Then** system displays list of all usage and management activities<br>
**When** user selects an activity<br>
**Then** system displays details for the selected activity<br>
**When** user navigates back<br>
**And** user selects 'About Organization'<br>
**Then** system displays information about the organization<br>

---

### LTC23

#### View card-specific activity list

**Given** user has completed PID setup and opened the app<br>
**When** user selects 'activities' on card details screen<br>
**Then** system displays list of activities related to the selected card<br>
**When** user selects an activity<br>
**Then** system displays details for the selected activity<br>
**When** user navigates back<br>
**And** user selects 'About Organization'<br>
**Then** system displays organization information<br>

---

### LTC24

#### View all available cards

**Given** user has completed PID setup and unlocked the app<br>
**When** user completes one of the relevant flows (unlock app, obtain PID, or
obtain card)<br>
**Then** system displays all cards currently available in the app<br>

---

### LTC25

#### View Card Details

**Given** dashboard is opened<br>
**When** user selects a card<br>
**Then** system displays the card details<br>
**When** user selects card attributes<br>
**Then** system displays the card attributes<br>
**When** user navigates back<br>
**And** user selects card history<br>
**Then** system displays the card history<br>
**When** user navigates back<br>
**And** user selects organization<br>
**Then** system displays the organization<br>

---

### LTC26

#### Show app menu

**Given** user has completed PID setup and opened the app<br>
**When** user selects 'menu' on the dashboard<br>
**Then** system displays all app menu items<br>

---

### LTC27

#### View settings menu

**Given** app menu is shown<br>
**When** user selects 'Settings'<br>
**Then** system displays settings menu items<br>
**When** user navigates back<br>
**Then** system displays app menu items<br>

---

### LTC28

#### View app information

**Given** user selects app information<br>
**Then** system displays the About App screen<br>

---

### LTC29

#### View privacy statement

**Given** About App screen is shown<br>
**When** user selects 'Privacy Statement'<br>
**Then** system displays the Privacy Statement screen<br>
**When** user navigates back<br>
**Then** system displays the About App screen again<br>

---

### LTC30

#### Select a new language

**Given** system displays language selection screen<br>
**When** user selects a non-active language<br>
**Then** system updates the UI to the selected language<br>
**And** displays the updated language as active<br>
**When** user navigates back<br>
**Then** system displays the settings menu in the selected language<br>

---

### LTC31

#### Get help

**Given** system displays a PIN screen<br>
**When** user selects 'help'<br>
**Then** system displays get help screen<br>

---

### LTC32

#### Open closed app

**Given** the app is installed<br>
**And** wallet is registered<br>
**When** user opens the app<br>
**Then** system displays splash screen<br>
**And** system determines if wallet is registered<br>
**And** system displays the dashboard<br>

---

### LTC33

#### Open app via universal link

**Given** the app is installed<br>
**And** wallet is registered<br>
**When** user opens the app by following a universal link<br>
**Then** system validates the universal link<br>
**And** app is opened<br>

---

### LTC34

#### Open app with unregistered wallet

**Given** the app is installed<br>
**When** user opens the app<br>
**Then** system displays introduction screen<br>

---

### LTC35

####  Invoke universal link when app is not installed

**Given** user invokes a universal link<br>
**And** the app is not installed<br>
**Then** system navigates user to fallback page<br>
**And** system displays message to install the app<br>

---

### LTC36

#### Open universal link via external QR scanner

**Given** user invokes a universal link using an external QR scanner<br>
**Then** system displays message to rescan the QR code using the in-app scanner<br>

---

### LTC37

#### Unlock app with correct PIN

**Given** user has completed setup of remote PIN and biometrics<br>
**When** user opens the app<br>
**Then** system displays PIN screen<br>
**When** user enters correct PIN<br>
**Then** system displays dashboard<br>

---

<!-- Manual  -->

### LTC38

#### Unlock app with biometric

**Given** user has completed setup of remote PIN and biometrics<br>
**When** user opens the app<br>
**Then** system requests biometric<br>
**When** user enters valid biometric<br>
**Then** system displays dashboard<br>

---

<!-- Manual  -->

### LTC39

#### Unlock app with invalid biometric

**Given** user has completed setup of remote PIN and biometrics<br>
**When** user opens the app<br>
**Then** system requests biometric<br>
**When** user enters invalid biometric<br>
**Then** device gives option to try again<br>

---

### LTC40

#### Unlock app with invalid PIN

**Given** user has completed setup of remote PIN and biometrics<br>
**When** user opens the app<br>
**Then** system displays PIN screen<br>
**When** user enters invalid PIN<br>
**Then** system handles it according to PIN retry policy<br>

---

### LTC41

#### Recover PIN

**Given** user start PIN recovery<br>
**When** user changes PIN successfully<br>
**Then** user can use new PIN<br>
**And** user can not use old PIN<br>

---

### LTC42

#### App update is available

**Given** the app is installed<br>
**And** an app update is available<br>
**When** user opens the app<br>
**Then** System displays a message on update informing user that an update
is available and offers instructions on how to update<br>

---

### LTC43

#### Current app version is blocked

**Given** the app is installed<br>
**And** an app update is available<br>
**And** current app version is blocked<br>
**When** user opens the app<br>
**Then** System displays a message on update informing user that current app
version is blocked and offers instructions on how to update<br>

---

### LTC44

#### Wallet not created when universal link is invoked

**Given** user invokes a universal link<br>
**And** wallet is not created<br>
**Then** System shows introduction screen<br>

---

### LTC45

#### Change PIN

**When** user changes the PIN code<br>
**Then** the change is successful<br>
**And** old PIN is unusable<br>
**And** new PIN is usable<br>

---

### LTC46

#### PIN is invalid timeout

**Scenario Outline:**<br>
**Given** system allows 4 rounds of 4 attempts each<br>
**And** user enters their PIN invalid for all `<Y>` attempts in round
`<round>`<br>
**Then** system introduces a timeout of `<Z{i}>` for that round<br>

**Examples:**

| round | Z{i} |
| ----- | ---- |
| 1     | 1m   |
| 2     | 5m   |
| 3     | 60m  |

---

### LTC47

#### PIN is invalid Block

**Given** system allows 4 rounds of 4 attempts each<br>
**And** user enters their PIN invalid for all attempts in all 4
rounds<br>
**Then** system blocks access<br>

---

### LTC48

#### PIN entries do not match, try again

**Given** user enters the correct current PIN<br>
**When** user does an invalid confirmation<br>
**Then** system displays a message that the PIN entries are not equal and
offers user to try again<br>

---

### LTC49

#### PIN entries do not match, choose new PIN

**Given** user enters PIN<br>
**When** user does an invalid confirmation<br>
**Then** system displays a message that the PIN entries are not equal and
instructs user to choose a new PIN<br>

---

### LTC50

#### PIN entry does not conform to policy

**Scenario Outline:**<br>
**Given** user enters the correct current PIN<br>
**When** user enters a PIN `<pin>` that does not conform to policy<br>
**Then** system displays a message that the PIN entry is not conformant and
instructs user to choose a new PIN<br>

**Examples:**

| pin    |
| ------ |
| 111111 |
| 123456 |
| 654321 |

---

### LTC51

#### Setup PIN

**Given** user completed introduction<br>
**When** user enters a valid pin<br>
**And** user confirms pin<br>
**Then** remote pin is configured<br>

---

<!-- Manual  -->

### LTC52

#### Setup PIN fails device does not pass app and key attestation

**Given** device can not pass app and key attestation<br>
**When** user sets up a remote PIN<br>
**Then** System displays message that device is not supported<br>

---

### LTC53

#### Logout from menu

**Given** user has completed PID setup and opened the app<br>
**When** user selects 'Logout' from the menu<br>
**Then** system logs out user<br>
**And** displays the PIN screen<br>

---

### LTC54

#### Logout due to inactivity

**Given** user is inactive for warning timeout Z<br>
**Then** system displays inactivity prompt<br>
**When** user remains inactive for X - Z minutes<br>
**Then** system logs out user<br>
**And** displays the PIN screen<br>

---

### LTC55

#### Logout due to background timeout

**Given** user puts the app in the background<br>
**When** background timeout Y elapses<br>
**Then** system logs out user<br>
**And** app remains in the background<br>

---

### LTC56

#### Confirm logout on inactivity prompt

**Given** system displays inactivity prompt<br>
**When** user selects 'Log out'<br>
**Then** system logs out user<br>
**And** displays the PIN screen<br>

---

### LTC57

#### Dismiss inactivity prompt

**Given** system displays inactivity prompt<br>
**When** user selects 'Yes, continue'<br>
**Then** prompt is dismissed<br>
**And** system displays the currently active screen<br>

---

<!-- Manual  -->

### LTC58

#### Skip setting up biometrics

**Given** user has set up pin<br>
**And** device supports biometrics<br>
**When** user skips setting up biometrics<br>
**Then** system displays message wallet is secured by pin<br>

---

<!-- Manual  -->

### LTC59

#### Device does not support biometrics

**Given** device does not support biometrics<br>
**When** user open apps settings menu<br>
**Then** system does not display biometric configuration option<br>

---

<!-- Manual  -->

### LTC60

#### Disable biometrics

**Given** user has enabled biometrics<br>
**When** user disables biometrics<br>
**Then** biometrics is disabled<br>
**And** user can not use biometric login<br>

---

<!-- Manual  -->

### LTC61

#### Setup biometrics in settings

**Given** user has not enabled biometrics<br>
**And** device supports biometrics<br>
**When** user enables biometrics<br>
**Then** system requests pin<br>
**When** user enters pin<br>
**And** user enter biometric<br>
**Then** biometrics is enabled<br>
**And** user can use biometric login<br>

---

<!-- Manual  -->

### LTC62

#### Transfer wallet

**Given** user has an existing active source wallet<br>
**And** user has completed PID issuance on its destination wallet<br>
**When** user completes wallet transfer<br>
**Then** the destination wallet contains the data of the source wallet<br>
**And** the destination wallet is active<br>
**And** the source wallet does not contain data<br>
**And** the source wallet is inactive<br>

---

<!-- Manual  -->

### LTC63

#### Stop transfer flow on source device

**Given** user has an existing active source wallet<br>
**And** user has completed PID issuance on its destination wallet<br>
**When** user aborts wallet transfer on the source device<br>
**Then** the source wallet displays the dashboard screen<br>
**And** user is prompted on the destination device to try again<br>

---

<!-- Manual  -->

### LTC64

#### Stop transfer flow on destination device

**Given** user has an existing active source wallet<br>
**And** user has completed PID issuance on its destination wallet<br>
**When** user aborts wallet transfer on the destination device<br>
**Then** user is prompted on the destination device to try again<br>
**And** the source wallet displays the dashboard screen<br>

---

### LTC65

#### Select another card to be disclosed

**Given** user has multiple cards that can fulfill a disclosure request<br>
**When** user starts disclosure<br>
**Then** system offers user option to select another card<br>
**When** user selects a card<br>
**Then** the requested attributes are disclosed from the previously selected card<br>

---

### LTC66

#### Renew PID

**Given** user has issued PID<br>
**When** user renews PID<br>
**Then** PID is renewed<br>
**And** a card renewal event is added to the history<br>

---

### LTC67

#### Revoke PID

**Given** user has issued PID<br>
**When** PID is revoked<br>
**Then** PID card is displayed as revoked<br>
**And** PID cannot be presented anymore<br>

---

### LTC68

#### Revoke EAA Card

**Given** user has an EAA card in its wallet<br>
**When** EAA card is revoked<br>
**Then** EAA card is displayed as revoked<br>
**And** EAA card cannot be presented anymore<br>

---

### LTC69

#### Universal link is invoked while wallet is not personalized

**Given** user is performing app personalization<br>
**When** user invokes a universal link<br>
**Then** System displays message informing the user that personalization should be completed first<br>

---

### LTC70

#### Receive revocation code

**Given** user has completed remote PIN setup<br>
**Then** system display a revocation code<br>
**And** user has to confirm the revocation code is written to continue<br>

---

### LTC71

#### System sends notifications for card status changes

**Given** a <card_type> card exists in the wallet<br>
**And** the card <scenario><br>
**Then** the system displays an in-app notification at <schedule><br>
**And** the system sends a push notification at <schedule><br>
**And** the in-app notification contains "<in_app_message>"<br>
**And** the push notification contains "<push_message>"<br>
**When** the user selects the notification<br>
**Then** the system displays the card details screen<br>

Cases:

| card_type | scenario                      | schedule   | in_app_message                                     | push_message                                                              |
|-----------|-------------------------------|------------|----------------------------------------------------|---------------------------------------------------------------------------|
| PID       | will expire in 14 days        | 10:00 AM   | In 14 days, your app will not be working           | In 14 days, your app will not working. Solve this in the app.             |
| PID       | has expired                   | immediate  | Attention: your NL Wallet is not working right now | Attention: your NL Wallet does not work right now. Solve this in the app. |
| PID       | is revoked by issuer          | immediate  | Attention: your NL Wallet is not working right now | Attention: your NL Wallet does not work right now. Solve this in the app. |
| EAA       | will expire in 14 days        | 10:00 AM   | {Card} expires in 14 days                          | {Card} expires in 14 days. Replace this card if you still need it.        |
| EAA       | has expired                   | immediate  | {Card} expired                                     | {Card} expired. Replace this card if you still need it.                   |
| EAA       | is revoked by issuer          | immediate  | {Card} withdrawn by issuer                         | {Card} withdrawn by issuer. Replace this card if you still need it.       |

---

### LTC72

#### Configure notifications

**Given** notifications are turned off<br>
**And** OS notifications are not scheduled<br>
**When** user enables notifications<br>
**Then** OS notifications are scheduled<br>

---

### LTC73

#### View revocation code in settings

**Given** user has completed PID setup and opened the app<br>
**When** user selects 'View your deletion code' from the settings menu<br>
**And** use confirms with PIN to view the revocation code<br>
**Then** system displays the revocation code<br>
