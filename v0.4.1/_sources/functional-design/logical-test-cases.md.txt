# Logical Test Cases

## LTCs

This page describes the logical test cases (LTCs) that are used to verify the correct functionality of the wallet app.
Test are written in Gherkin syntax with a Given When Then structure. Each LTC is implemented with at least one manual 
or automated tests.

### LTC1

#### PID issuance

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

#### Authentication at auth server fails

**Given** user has completed security setup  
**And** user authentication at auth server fails  
**Then** system displays message that authentication failed  
**And** provides a link to try again

---

### LTC4

#### Rejects issued attributes  

**Given** user has completed security setup  
**And** user authenticates at auth server  
**And** system displays issued attributes to user for verification  
**When** user rejects issued attributes  
**Then** provides a link to try again

---

### LTC5

#### Disclosure based Issuance

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

---

<!-- Manual  -->

### LTC7

#### Cross-device generic issuance

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

#### Reject disclosure of attributes

**Given** user starts card issuance  
**When** system asks for disclosure consent  
**And** user selects 'stop'  
**Then** system confirms cancellation

---

### LTC9

#### No cards to be issued

**Given** user has no cards to be issued available at EAA issuer
**When** user performs disclosure based issuance to retrieve cards  
**Then** system displays error message that no cards are available for issuance

---

### LTC10

#### Wallet does not contain requested attributes

**Given** wallet does not contain attributes to fulfill a disclosure request
from an issuer  
**When** user invokes a universal link from a (Q)EAA issuer  
**Then** System displays an error message with instructions  
**When** user selects 'see details'  
**Then** system displays a bottom sheet with app information

---

### LTC11

#### Renew card

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

### LTC12

#### View introduction

**Given** the app is opened   
**And** user has not completed introduction   
**When** user navigates through the introduction screens     
**Then** system displays set pin screen  

---

### LTC13

#### Skip introduction

**Given** the app is opened  
**And** user has not completed introduction  
**When** user skips the introduction screens     
**Then** systems displays the privacy introduction screen  

---

### LTC14

#### View app tour

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

### LTC15

#### Share data

**Given** user has completed PID setup and opened the app  
**When** user starts disclosure process at relying party  
**Then** system requests user consent  
**When** user approves with PIN  
**Then** system discloses attributes to relying party  
**And** system displays data shared message

---

<!-- Manual  --> 

### LTC16

#### Cross-device share data

**Given** user starts disclosure on a non-mobile device  
**When** user scans QR code with wallet  
**Then** system validates relying party URL  
**When** user confirms to proceed  
**And** approves disclosure with PIN  
**Then** system completes disclosure  
**And** displays a success message

---

### LTC17

#### Decline consent to share data

**Given** user is shown consent screen  
**When** user selects 'Stop'  
**Then** system confirms cancellation  
**When** user confirms  
**Then** system displays the stopped screen  
**And** provides a link to the dashboard

---

### LTC18

#### RP Login

**Given** user has completed PID setup and opened the app  
**When** user starts login process at relying party  
**Then** system requests consent to disclose BSN  
**When** user approves with PIN  
**Then** system discloses BSN to relying party  
**And** relying pary displays login success message

---

<!-- Manual  --> 

### LTC19

#### Cross-device login

**Given** user starts login on a non-mobile device  
**When** user scans QR code with wallet  
**Then** system validates relying party URL  
**When** user confirms to continue  
**And** approves disclosure with PIN  
**Then** login is completed  
**And** system displays success message

---

### LTC20

#### Disclosure fails

**Given** user has completed security setup  
**When** user starts a disclosure flow  
**And** disclosure fails  
**And** system displays message disclosure failed  
**And** provides a link to try again

---

### LTC21

#### Delete App data

**Given** user has completed PID setup and opened the app  
**When** user selects 'Remove data' from the settings menu  
**Then** system displays a prompt with 'Cancel' and 'Yes, Delete' options  
**When** user selects 'Yes, Delete'  
**Then** system displays the introduction screen

---

### LTC22

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

### LTC23

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

### LTC24

#### View all available cards

**Given** user has completed PID setup and unlocked the app  
**When** user completes one of the relevant flows (unlock app, obtain PID, or
obtain card)  
**Then** system displays all cards currently available in the app

---

### LTC25

#### View Card Details

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

### LTC26

#### Show app menu

**Given** user has completed PID setup and opened the app  
**When** user selects 'menu' on the dashboard  
**Then** system displays all app menu items

---

### LTC27

#### View settings menu

**Given** app menu is shown  
**When** user selects 'Settings'  
**Then** system displays settings menu items  
**When** user navigates back  
**Then** system displays app menu items

---

### LTC28

#### View app information 

**Given** user selects app information  
**Then** system displays the About App screen

---

### LTC29

#### View privacy statement

**Given** About App screen is shown  
**When** user selects 'Privacy Statement'  
**Then** system displays the Privacy Statement screen  
**When** user navigates back  
**Then** system displays the About App screen again

---

### LTC30

#### Select a new language

**Given** system displays language selection screen  
**When** user selects a non-active language  
**Then** system updates the UI to the selected language  
**And** displays the updated language as active  
**When** user navigates back  
**Then** system displays the settings menu in the selected language

---

### LTC31

#### Get help   

**Given** system displays a PIN screen  
**When** user selects 'help'    
**Then** system displays get help screen

---

### LTC32

#### Open closed app 

**Given** the app is installed   
**And** wallet is registered  
**When** user opens the app  
**Then** system displays splash screen  
**And** system determines if wallet is registered  
**And** system displays the dashboard

---

### LTC33

#### Open app via universal link

**Given** the app is installed  
**And** wallet is registered  
**When** user opens the app by following a universal link  
**Then** system validates the universal link  
**And** app is opened  

---

### LTC34

#### Open app with unregistered wallet

**Given** the app is installed  
**When** user opens the app  
**Then** system displays introduction screen

---

### LTC35

####  Invoke universal link when app is not installed 

**Given** user invokes a universal link  
**And** the app is not installed  
**Then** system navigates user to fallback page  
**And** system displays message to install the app

---

### LTC36

#### Open universal link via external QR scanner

**Given** user invokes a universal link using an external QR scanner  
**Then** system displays message to rescan the QR code using the in-app scanner

---

### LTC37

#### Unlock app with correct PIN

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system displays PIN screen  
**When** user enters correct PIN  
**Then** system displays dashboard

---

<!-- Manual  --> 

### LTC38

#### Unlock app with biometric

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system requests biometric  
**When** user enters valid biometric  
**Then** system displays dashboard

---

<!-- Manual  -->

### LTC39

#### Unlock app with invalid biometric

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system requests biometric  
**When** user enters invalid biometric  
**Then** device gives option to try again

---

### LTC40

#### Unlock app with invalid PIN

**Given** user has completed setup of remote PIN and biometrics  
**When** user opens the app  
**Then** system displays PIN screen  
**When** user enters invalid PIN  
**Then** system handles it according to PIN retry policy

---

### LTC41

#### Recover PIN

**Given** user start PIN recovery  
**When** user changes PIN successfully  
**Then** user can use new PIN  
**And** user can not use old PIN

---

### LTC42

#### App update is available

**Given** the app is installed  
**And** an app update is available  
**When** user opens the app  
**Then** System displays a message on update informing user that an update
is available and offers instructions on how to update

---

### LTC43

#### Current app version is blocked

**Given** the app is installed  
**And** an app update is available  
**And** current app version is blocked  
**When** user opens the app  
**Then** System displays a message on update informing user that current app
version is blocked and offers instructions on how to update

---

### LTC44

#### Wallet not created when universal link is invoked

**Given** user invokes a universal link  
**And** wallet is not created  
**Then** System shows introduction screen

---

### LTC45

#### Change PIN

**When** user changes the PIN code  
**Then** the change is successful  
**And** old PIN is unusable  
**And** new PIN is usable  

---

### LTC46

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

### LTC47

#### PIN is invalid Block

**Given** system allows 4 rounds of 4 attempts each  
**And** user enters their PIN invalid for all attempts in all 4
rounds  
**Then** system blocks access

---

### LTC48

#### PIN entries do not match, try again

**Given** user enters the correct current PIN  
**When** user does an invalid confirmation  
**Then** system displays a message that the PIN entries are not equal and
offers user to try again

---

### LTC49

#### PIN entries do not match, choose new PIN

**Given** user enters PIN  
**When** user does an invalid confirmation  
**Then** system displays a message that the PIN entries are not equal and
instructs user to choose a new PIN

---

### LTC50

#### PIN entry does not conform to policy

**Scenario Outline:**  
**Given** user enters the correct current PIN  
**When** user enters a PIN `<pin>` that does not conform to policy  
**Then** system displays a message that the PIN entry is not conformant and
instructs user to choose a new PIN

**Examples:**

| pin    |
| ------ |
| 111111 |
| 123456 |
| 654321 |

---

### LTC51

#### Setup PIN

**Given** user completed introduction  
**When** user enters a valid pin  
**And** user confirms pin  
**Then** remote pin is configured

---

<!-- Manual  -->

### LTC52

#### Setup PIN fails device does not pass app and key attestation

**Given** device can not pass app and key attestation  
**When** user sets up a remote PIN  
**Then** System displays message that device is not supported

---

### LTC53

#### Logout from menu

**Given** user has completed PID setup and opened the app  
**When** user selects 'Logout' from the menu  
**Then** system logs out user  
**And** displays the PIN screen

---

### LTC54

#### Logout due to inactivity

**Given** user is inactive for warning timeout Z  
**Then** system displays inactivity prompt  
**When** user remains inactive for X - Z minutes  
**Then** system logs out user  
**And** displays the PIN screen

---

### LTC55

#### Logout due to background timeout

**Given** user puts the app in the background  
**When** background timeout Y elapses  
**Then** system logs out user  
**And** app remains in the background

---

### LTC56

#### Confirm logout on inactivity prompt

**Given** system displays inactivity prompt  
**When** user selects 'Log out'  
**Then** system logs out user  
**And** displays the PIN screen

---

### LTC57

#### Dismiss inactivity prompt

**Given** system displays inactivity prompt  
**When** user selects 'Yes, continue'  
**Then** prompt is dismissed  
**And** system displays the currently active screen

---

<!-- Manual  --> 

### LTC58

#### Skip setting up biometrics

**Given** user has set up pin  
**And** device supports biometrics  
**When** user skips setting up biometrics  
**Then** system displays message wallet is secured by pin

---

<!-- Manual  -->

### LTC59

#### Device does not support biometrics

**Given** device does not support biometrics  
**When** user open apps settings menu  
**Then** system does not display biometric configuration option

---

<!-- Manual  --> 

### LTC60

#### Disable biometrics

**Given** user has enabled biometrics  
**When** user disables biometrics  
**Then** biometrics is disabled  
**And** user can not use biometric login

---

<!-- Manual  --> 

### LTC61

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

<!-- Manual  --> 

### LTC62

#### Transfer wallet 

**Given** user has an existing active source wallet  
**And** user has completed PID issuance on its destination wallet  
**When** user completes wallet transfer  
**Then** the destination wallet contains the data of the source wallet  
**And** the destination wallet is active  
**And** the source wallet does not contain data  
**And** the source wallet is inactive

---

<!-- Manual  --> 

### LTC63

#### Stop transfer flow on source device

**Given** user has an existing active source wallet  
**And** user has completed PID issuance on its destination wallet  
**When** user aborts wallet transfer on the source device  
**Then** the source wallet displays the dashboard screen  
**And** user is prompted on the destination device to try again  

---

<!-- Manual  -->

### LTC64

#### Stop transfer flow on destination device

**Given** user has an existing active source wallet  
**And** user has completed PID issuance on its destination wallet  
**When** user aborts wallet transfer on the destination device  
**Then** user is prompted on the destination device to try again  
**And** the source wallet displays the dashboard screen

---

### LTC65

#### Select another card to be disclosed

**Given** user has multiple cards that can fulfill a disclosure request  
**When** user starts disclosure  
**Then** system offers user option to select another card    
**When** user selects a card  
**Then** the requested attributes are disclosed from the previously selected card

---

### LTC66

#### Renew PID 

**Given** user has issued PID
**When** user renews PID  
**Then** PID is renewed   
**And** a card renewal event is added to the history
