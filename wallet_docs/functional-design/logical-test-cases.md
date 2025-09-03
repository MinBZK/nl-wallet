# Logical Test Cases

## Issuance

### LTC1

#### PID issuance happy flow

**Given** user has completed security setup  
**When** user authenticates at auth server  
**Then** system displays issued attributes to user for verification  
**When** user adds attributes  
**And** confirms using their PIN  
**Then** system displays message that wallet is created  
**And** provides a link to the dashboard

---

### LTC2

#### Issuance fails

**Given** user has completed security setup  
**And** user authenticates at auth server  
**When** issuance fails  
**Then** system displays message that issuance failed  
**And** provides a link to try again

---

### LTC3

#### Authentication with auth server fails

**Given** user has completed security setup  
**And** user authentication at auth server fails  
**Then** system displays message that authentication failed  
**And** provides a link to try again

---

### LTC4

#### User rejects issued attributes

**Given** user has completed security setup  
**When** user authenticates at auth server  
**Then** system displays issued attributes to user for verification

---

### LTC5

#### Disclosure based Issuance happy flow

**Given** user has completed PID setup and opened the app  
**When** user invokes a universal link from a (Q)EAA issuer  
**Then** system requests approval for disclosure  
**When** user approves with PIN  
**Then** system validates PIN and proceeds  
**And** system requests approval for attestations  
**When** user approves with PIN  
**Then** system issues card and displays it to user  
**And** provides a link to the dashboard


---

### LTC6

#### Invalid universal link

**Given** user invokes an invalid universal link  
**Then** system informs user link is not recognized  
**And** provides a link to try again

---

### LTC7

#### cross-device generic issuance

**Given** user is on a device with wallet installed  
**When** relying party presents a QR code  
**And** user scans it with wallet  
**Then** system requests approval for disclosure  
**When** user approves with PIN  
**Then** system validates PIN and proceeds  
**And** system requests approval for attestations  
**When** user approves with PIN  
**Then** system issues card and displays it to user  
**And** provides a link to the dashboard


---

### LTC8

#### User rejects disclosure of attributes

**Given** user starts card issuance  
**When** system asks for disclosure consent  
**And** user selects 'stop'  
**Then** system confirms cancellation

---

### LTC9

#### User continues after initially cancelling disclosure

**Given** user starts disclosure  
**And** selects 'stop'  
**When** system asks confirmation  
**And** user selects 'no'  
**Then** disclosure flow continues

---

### LTC10

#### User continues after initially cancelling issuance

**Given** user starts issuance  
**And** selects 'stop'  
**When** system asks confirmation  
**And** user selects 'no'  
**Then** issuance flow continues

---

### LTC11

#### wallet does not contain requested attributes

**Given** wallet does not contain attributes to fulfill a disclosure request
from an issuer  
**When** user invokes a universal link from a (Q)EAA issuer  
**Then** System displays an error message with instructions  
**When** user selects 'see details'  
**Then** system displays a bottom sheet with app information

---

### LTC12

#### Renew card happy flow

**Given** user has an EAA card in its wallet  
**When** user invokes a universal link from issuer of card  
**Then** system requests approval for disclosure  
**When** user approves with PIN  
**Then** system validates PIN and proceeds  
**And** system requests approval card renewal  
**When** user approves with PIN  
**Then** system renews card  
**And** provides a link to the dashboard  
**And** merges history of old and new cards  
**And** adds a card renewel event to the history

---

## Introduction

### LTC13

#### Introduction Happy flow

**Given** the app is opened  
**And** user has not completed introduction 
**When** user navigates through the introduction screens    
**Then** system displays set pin screen

---

### LTC14

#### User skips introduction

**Given** the app is opened  
**And** user has not completed introduction
**When** user skips the introduction screens   
**Then** systems displays the privacy introduction screen

---

### LTC15

#### User navigates back to previous introduction screen

**Given** user has navigated forward to an introduction screen  
**And** user navigates back  
**Then** systems displays the previous introduction screen

---

### LTC16

#### User views app tour

**Given** user has not closed app after obtaining PID  
**When** user views the app dashboard  
**Then** system displays a non-dismissible banner    
**When** user views the menu  
**Then** system displays an app tour menu item  
**When** user selects app tour  
**Then** system displays an app tour overview with a list of video items  
**When** user selects a video  
**Then** system opens the videoplayer  
**And** videoplayer contains correct controls  

---

## Disclosure

### LTC17

#### Share data happy flow

**Given** user has completed PID setup and opened the app  
**When** user starts disclosure process at relying party  
**Then** system requests user consent  
**When** user approves with PIN  
**Then** system discloses attributes to relying party  
**And** system displays data shared message

---

### LTC18

#### Cross-device share data

**Given** user starts disclosure on a non-mobile device  
**When** user scans QR code with wallet  
**Then** system validates relying party URL  
**When** user confirms to proceed  
**And** approves disclosure with PIN  
**Then** system completes disclosure  
**And** displays a success message

---

### LTC19

#### User does not give consent to share data

**Given** user is shown consent screen  
**When** user selects 'Stop'  
**Then** system confirms cancellation  
**When** user confirms  
**Then** system displays the stopped screen  
**And** provides a link to the dashboard

---

### LTC20

#### User stops at Share data approve organization

**Given** user is shown approve organization screen  
**When** user selects 'Stop'  
**Then** system confirms cancellation  
**When** user confirms  
**Then** system displays the stopped screen  
**And** provides a link to the dashboard

---

### LTC21

#### User continues after initially cancelling

**Given** user selects 'Stop'  
**When** system displays message conforming cancellation  
**And** user selects 'No'  
**Then** disclosure flow continues

---

### LTC22

#### invalid universal link in QR

**Given** user scans a QR with an invalid link  
**Then** system displays invalid QR message

---

### LTC23

#### RP Login happy flow

**Given** user has completed PID setup and opened the app  
**When** user starts login process at relying party  
**Then** system requests consent to disclose BSN  
**When** user approves with PIN  
**Then** system discloses BSN to relying party  
**And** relying pary displays login success message

---

### LTC24

#### Cross-device login

**Given** user starts login on a non-mobile device  
**When** user scans QR code with wallet  
**Then** system validates relying party URL  
**When** user confirms to continue  
**And** approves disclosure with PIN  
**Then** login is completed  
**And** system displays success message

---

### LTC25

#### User continues after initially cancelling

**Given** user selects 'Stop'  
**When** system displays message confirm cancellation  
**And** user selects 'No'  
**Then** login flow continues

---

### LTC26

#### Disclosure fails

**Given** user has completed security setup  
**When** user starts a disclosure flow  
**And** disclosure fails  
**And** system displays message disclosure failed  
**And** provides a link to try again

---

### LTC27

#### Wallet does not contain requested attributes

**Given** wallet does not contain attributes to fulfill a disclosure request
from a relying party  
**When** user invokes a universal link from a relying party  
**Then** System displays an error message with instructions

---

## Cards, App Data & Settings

### LTC28

#### Delete App data

**Given** user has completed PID setup and opened the app  
**When** user selects 'Remove data' from the settings menu  
**Then** system displays a prompt with 'Cancel' and 'Yes, Delete' options  
**When** user selects 'Yes, Delete'  
**Then** system displays the introduction screen

---

### LTC29

#### Cancel app data deletion

**Given** system displays confirm delete message  
**When** user selects 'Cancel'  
**Then** system displays the settings menu

---

### LTC30

#### View activity list

**Given** user has completed PID setup and opened the app  
**When** user selects 'activities' from settings or dashboard  
**Then** system displays list of all usage and management activities  
**When** user selects an activity  
**Then** system displays details for the selected activity  
**When** user navigates back  
**And** user selects 'About Organization'  
**Then** system displays information about the organization

---

### LTC31

#### View card-specific activity list

**Given** user has completed PID setup and opened the app  
**When** user selects 'activities' on card details screen  
**Then** system displays list of activities related to the selected card  
**When** user selects an activity  
**Then** system displays details for the selected activity  
**When** user navigates back  
**And** user selects 'About Organization'  
**Then** system displays organization information

---

### LTC32

#### Show all available cards

**Given** user has completed PID setup and unlocked the app  
**When** user completes one of the relevant flows (unlock app, obtain PID, or
obtain card)  
**Then** system displays all cards currently available in the app

---

### LTC33

#### Show Card Details

**Given** dashboard is opened  
**When** user selects a card  
**Then** system displays the card details  
**When** user selects card attributes  
**Then** system displays the card attributes  
**When** user navigates back  
**And** user selects card history  
**Then** system displays the card history  
**When** user navigates back  
**And** user selects organization  
**Then** system displays the organization

---

### LTC34

#### Show app menu

**Given** user has completed PID setup and opened the app  
**When** user selects 'menu' on the dashboard  
**Then** system displays all app menu items

---

### LTC35

#### Settings menu

**Given** app menu is shown  
**When** user selects 'Settings'  
**Then** system displays settings menu items  
**When** user navigates back  
**Then** system displays app menu items

---

### LTC36

#### Show app information from menu

**Given** user has completed PID setup and opened the app  
**When** user selects 'about this app' from the menu  
**Then** system displays the About App screen

---

### LTC37

#### View privacy statement

**Given** About App screen is shown  
**When** user selects 'Privacy Statement'  
**Then** system displays the Privacy Statement screen  
**When** user navigates back  
**Then** system displays the About App screen again

---

### LTC38

#### Show language selection screen

**Given** user has completed PID setup and opened the app  
**When** user selects 'Change Language' in the settings menu  
**Then** system displays the language selection screen  
**And** displays the currently active language

---

### LTC39

#### Select a new language

**Given** system displays language selection screen  
**When** user selects a non-active language  
**Then** system updates the UI to the selected language  
**And** displays the updated language as active  
**When** user navigates back  
**Then** system displays the settings menu in the selected language

---

### LTC40

#### Show app information on introduction screen

**Given** system displays an introduction screen  
**When** user selects 'see details'  
**Then** system displays a bottom sheet with app information

---

### LTC41

#### Show app information on PIN screen

**Given** system displays a PIN screen  
**When** user selects 'see details'  
**Then** system displays a bottom sheet with app information

---

### LTC79

#### Get help on PIN screen  

**Given** system displays a PIN screen  
**When** user selects 'help'    
**Then** system displays get help screen  
**When** user selects back  
**Then** system displays a PIN screen

---

## App lock & Security

### LTC42

#### Open closed app 

**Given** the app is installed   
**And** wallet is registered  
**When** user opens the app  
**Then** system displays splash screen  
**And** system determines if wallet is registered  
**And** system displays the dashboard

---

### LTC43

#### Open app via universal link

**Given** the app is installed  
**And** wallet is registered  
**When** user opens the app by following a universal link  
**Then** system validates the universal link  
**And** app is opened  
**And** system displays dashoboard

---

### LTC44

#### Wallet is not registered

**Given** the app is installed  
**When** user opens the app  
**Then** system displays introduction screen

---

### LTC45

#### App is not installed when universal link is invoked

**Given** user invokes a universal link  
**And** the app is not installed  
**Then** system navigates user to fallback page  
**And** system displays message to install the app

---

### LTC46

#### Universal link is opened via external QR scanner

**Given** user invokes a universal link using an external QR scanner  
**Then** system displays message to rescan the QR code using the in-app scanner

---

### LTC47

#### Unlock app with correct PIN

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system displays PIN screen  
**When** user enters correct PIN  
**Then** system displays dashboard

---

### LTC48

#### Unlock app with biometric

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system requests biometric  
**When** user enters valid biometric  
**Then** system displays dashboard

---

### LTC49

#### Unlock app with invalid biometric

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system requests biometric  
**When** user enters invalid biometric  
**Then** device gives option to try again

---

### LTC50

#### Unlock app with invalid PIN

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system displays PIN screen  
**When** user enters invalid PIN  
**Then** system handles it according to PIN retry policy

---

### LTC51

#### User selects forgot PIN

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**And** user selects 'Forgot PIN?'  
**Then** system displays forgot PIN screen  
**When** user selects 'Delete Wallet'  
**Then** user enters UC9.4 Wipe all app data

---

### LTC52

#### App update is available

**Given** the app is installed  
**And** an app update is available  
**When** user opens the app  
**Then** System displays a message on update informing the User that an update
is available and offers instructions on how to update

---

### LTC53

#### Current app version is blocked

**Given** the app is installed  
**And** an app update is available  
**And** current app version is blocked  
**When** user opens the app  
**Then** System displays a message on update informing the User that current app
version is blocked and offers instructions on how to update

---

### LTC54

#### Wallet not created when universal link is invoked

**Given** user invokes a universal link  
**And** wallet is not created  
**Then** System shows introduction screen

---

### LTC55

#### Invalid universal link 

**When** user invokes an invalid universal link  
**Then** System displays an error message that universal link could not be opened

---

### LTC56

#### PIN Change Happy flow

**When** the user changes the PIN code  
**Then** the change is successful

---

### LTC57

#### PIN is invalid timeout

**Scenario Outline:**  
**Given** system allows 4 rounds of 4 attempts each  
**And** user enters their PIN invalid for all `<Y>` attempts in round
`<round>`  
**Then** system introduces a timeout of `<Z{i}>` for that round

**Examples:**

| round | Z{i} |
| ----- | ---- |
| 1     | 1m   |
| 2     | 5m   |
| 3     | 60m  |

---

### LTC58

#### PIN is invalid Block

**Given** system allows 4 rounds of 4 attempts each  
**And** user enters their PIN invalid for all attempts in all 4
rounds  
**Then** system blocks access

---

### LTC59

#### Mixed PIN entry success and failure

**Scenario Outline:**  
**Given** system allows 4 rounds of 4 attempts each  
**And** user enters their PIN invalid for `<failures>` attempt(s) in
round `<round>`  
**And** enters their PIN in attempt `<success_attempt>` of the same
round  
**Then** system grants access without proceeding to timeout or block

**Examples:**

| round | failures | success_attempt |
| ----- | -------- | --------------- |
| 1     | 2        | 3               |
| 2     | 1        | 2               |
| 3     | 3        | 4               |

---

### LTC60

#### Timeout reset after successful PIN entry

**Scenario Outline:**  
**Given** the system allows 4 rounds of 4 attempts each  
**And** user fails to enter their PIN correctly in round `<round>`  
**And** the timeout `<Z{i}>` is introduced  
**And** user enters their PIN after the timeout  
**Then** app resets the timeout policy for subsequent rounds

**Examples:**

| round | Z{i} |
| ----- | ---- |
| 1     | 1m   |
| 2     | 5m   |
| 3     | 60m  |

---

### LTC61

#### PIN entries do not match, try again

**Given** the user enters the correct current PIN  
**When** the user does an invalid confirmation  
**Then** the system displays a message that the PIN entries are not equal and
offers the user to try again

---

### LTC62

#### PIN entries do not match, choose new PIN

**Given** the user PIN  
**When** the user does an invalid confirmation  
**Then** the system displays a message that the PIN entries are not equal and
instructs the user to choose a new PIN

---

### LTC63

#### PIN entry does not conform to policy

**Scenario Outline:**  
**Given** the user enters the correct current PIN  
**When** the user enters a PIN `<pin>` that does not conform to policy  
**Then** the system displays a message that the PIN entry is not conformant and
instructs the user to choose a new PIN

**Examples:**

| pin    |
| ------ |
| 111111 |
| 123456 |
| 654321 |

---

### LTC64

#### PIN change fails, could not reach server

**Given** the server is not reachable  
**When** the user changes the PIN code  
**Then** the system displays message that the PIN change has failed and offers to try
again

---

### LTC65

#### PIN change fails, no internet

**Given** there is no internet connection  
**When** the user changes the PIN code  
**Then** the system displays message that the PIN change has failed and offers to try
again

---

### LTC66

#### Setup PIN Happy flow

**Given** user completed introduction  
**When** user enters a valid pin  
**And** user confirms pin  
**Then** remote pin is configured

---

### LTC67

#### Setup PIN fails device does not pass app and key attestation

**Given** device can not pass app and key attestation  
**When** user sets up a remote PIN  
**Then** System displays message that device is not supported

---

### LTC68

#### User forgot PIN

**Given** PIN screen is shown  
**When** user selects forgot PIN  
**Then** System displays a link to delete wallet

---

### LTC69

#### Manual logout from menu

**Given** user has completed PID setup and opened the app  
**When** user selects 'Logout' from the menu  
**Then** system logs out the user  
**And** displays the PIN screen

---

### LTC70

#### Logout due to inactivity

**Given** user is inactive for warning timeout Z  
**Then** system displays inactivity prompt  
**When** user remains inactive for X - Z minutes  
**Then** system logs out the user  
**And** displays the PIN screen

---

### LTC71

#### Logout due to background timeout

**Given** user puts the app in the background  
**When** background timeout Y elapses  
**Then** system logs out the user  
**And** app remains in the background

---

### LTC72

#### User confirms logout on inactivity prompt

**Given** system displays inactivity prompt  
**When** user selects 'Log out'  
**Then** system logs out the user  
**And** displays the PIN screen

---

### LTC73

#### User dismisses inactivity prompt

**Given** system displays inactivity prompt  
**When** user selects 'Yes, continue'  
**Then** prompt is dismissed  
**And** system displays the currently active screen

---

### LTC74

#### User skips setting up biometrics

**Given** user has set up pin  
**And** device supports biometrics  
**When** user skips setting up biometrics  
**Then** system displays message wallet is secured by pin

---

### LTC75

#### Device does not support biometrics

**Given** device does not support biometrics  
**When** user open apps settings menu  
**Then** system does not display biometric configuration option

---

### LTC76

#### User disables biometrics

**Given** user has enabled biometrics  
**When** user disables biometrics  
**Then** biometrics is disabled  
**And** user can not use biometric login

---

### LTC77

#### Setup biometrics in settings

**Given** user has not enabled biometrics  
**And** device supports biometrics  
**When** user enables biometrics  
**Then** system requests pin  
**When** user enters pin  
**And** user enter biometric  
**Then** biometrics is enabled  
**And** user can use biometric login

---

### LTC78

#### User enters invalid current PIN

**Given** user has completed PID and opened the app  
**And** device supports biometrics  
**When** user enables biometrics  
**And** user enters correct biometric  
**And** user enters invalid PIN  
**Then** system handles it according to retry policy
