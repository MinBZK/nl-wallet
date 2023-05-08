# Flutter App Readme

This readme will specify how to run, build and configure the flutter app, which is currently mostly specified in the root level README.md.
Kickstarting this file with the deeplinks that can be used to trigger various mock scenarios.


### Deeplink Scenarios

Below you can find the deeplinks that can be used to trigger the supported mock scenarios.
On Android, the scenarios can be triggered from the command line by using `adb shell am start -a android.intent.action.VIEW -d "{deeplink}"`.
On iOS, the scenarios are triggered with the command `xcrun simctl openurl booted '{deeplink}'`
Note that the deeplinks only work on debug builds. For (mock) production builds you can generate a QR code from the content  and scan these using the app.

#### Issuance Scenarios

| Issue Scenario           | Content                                         | Deeplink                                                                                                    |
|--------------------------|-------------------------------------------------|-------------------------------------------------------------------------------------------------------------|
| Driving License          | {"id":"DRIVING_LICENSE","type":"issue"}         | walletdebuginteraction://deeplink#%7B%22id%22%3A%22DRIVING_LICENSE%22%2C%22type%22%3A%22issue%22%7D         |
| Extended Driving License | {"id":"DRIVING_LICENSE_RENEWED","type":"issue"} | walletdebuginteraction://deeplink#%7B%22id%22%3A%22DRIVING_LICENSE_RENEWED%22%2C%22type%22%3A%22issue%22%7D |
| Diploma                  | {"id":"DIPLOMA_1","type":"issue"}               | walletdebuginteraction://deeplink#%7B%22id%22%3A%22DIPLOMA_1%22%2C%22type%22%3A%22issue%22%7D               |
| Health Insurance         | {"id":"HEALTH_INSURANCE","type":"issue"}        | walletdebuginteraction://deeplink#%7B%22id%22%3A%22HEALTH_INSURANCE%22%2C%22type%22%3A%22issue%22%7D        |
| VOG                      | {"id":"VOG","type":"issue"}                     | walletdebuginteraction://deeplink#%7B%22id%22%3A%22VOG%22%2C%22type%22%3A%22issue%22%7D                     |
| Multiple Diplomas        | {"id":"MULTI_DIPLOMA","type":"issue"}           | walletdebuginteraction://deeplink#%7B%22id%22%3A%22MULTI_DIPLOMA%22%2C%22type%22%3A%22issue%22%7D           |

#### Verification Scenarios

| Verification Scenario     | Content                                           | Deeplink                                                                                                      |
|---------------------------|---------------------------------------------------|---------------------------------------------------------------------------------------------------------------|
| Job Application           | {"id":"JOB_APPLICATION","type":"verify"}          | walletdebuginteraction://deeplink#%7B%22id%22%3A%22JOB_APPLICATION%22%2C%22type%22%3A%22verify%22%7D          |
| Bar                       | {"id":"BAR","type":"verify"}                      | walletdebuginteraction://deeplink#%7B%22id%22%3A%22BAR%22%2C%22type%22%3A%22verify%22%7D                      |
| Marketplace Login         | {"id":"MARKETPLACE_LOGIN","type":"verify"}        | walletdebuginteraction://deeplink#%7B%22id%22%3A%22MARKETPLACE_LOGIN%22%2C%22type%22%3A%22verify%22%7D        |
| Car Rental                | {"id":"CAR_RENTAL","type":"verify"}               | walletdebuginteraction://deeplink#%7B%22id%22%3A%22CAR_RENTAL%22%2C%22type%22%3A%22verify%22%7D               |
| First Aid                 | {"id":"FIRST_AID","type":"verify"}                | walletdebuginteraction://deeplink#%7B%22id%22%3A%22FIRST_AID%22%2C%22type%22%3A%22verify%22%7D                |
| Confirm Appointment       | {"id":"CONFIRM_APPOINTMENT","type":"verify"}      | walletdebuginteraction://deeplink#%7B%22id%22%3A%22CONFIRM_APPOINTMENT%22%2C%22type%22%3A%22verify%22%7D      |
| Open Bank Account         | {"id":"OPEN_BANK_ACCOUNT","type":"verify"}        | walletdebuginteraction://deeplink#%7B%22id%22%3A%22OPEN_BANK_ACCOUNT%22%2C%22type%22%3A%22verify%22%7D        |
| Provide Contract Details  | {"id":"PROVIDE_CONTRACT_DETAILS","type":"verify"} | walletdebuginteraction://deeplink#%7B%22id%22%3A%22PROVIDE_CONTRACT_DETAILS%22%2C%22type%22%3A%22verify%22%7D |
| Create MonkeyBike Account | {"id":"CREATE_MB_ACCOUNT","type":"verify"}        | walletdebuginteraction://deeplink#%7B%22id%22%3A%22CREATE_MB_ACCOUNT%22%2C%22type%22%3A%22verify%22%7D        |

#### Sign Scenarios

| Sign Scenario    | Content                                 | Deeplink                                                                                            |
|------------------|-----------------------------------------|-----------------------------------------------------------------------------------------------------|
| Rental Agreement | {"id":"RENTAL_AGREEMENT","type":"sign"} | walletdebuginteraction://deeplink#%7B%22id%22%3A%22RENTAL_AGREEMENT%22%2C%22type%22%3A%22sign%22%7D |