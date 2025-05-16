# Wallet Web

The Wallet Web library provides a custom element that aims to be the easiest way to communicate with the NL Wallet Verification Server (`verification__server`), while providing a uniform user experience.

## Development

### Setup

Before one can build the library, the dependencies must be installed:
```sh
npm install
```

### Develop

To start a development server, with hot-reload, run:
```sh
npm run dev
```

### Build

To build the library, run:
```sh
npm install
npm run build
```

This will generate a `dist` folder with the following files:
- `nl-wallet-web.d.ts`
- `nl-wallet-web.iife.js`
- `nl-wallet-web.js`
- `nl-wallet-web.umd.cjs`

As the custom element is self-contained, the styles are included in the JavaScript files.

### Testing

Testing is done with [Vitest](https://vitest.dev/)
```sh
npm run test
```


## Usage

### Setup

For now, usage will be limited to the minimal setup. Therefore, a minimal `index.html` looks as follows:
```html
<html>
  <head>
    <script type="module" src="nl-wallet-web.iife.js"></script>
  </head>
  <body>
      <nl-wallet-button usecase="my_usecase"></nl-wallet-button>
  </body>
</html>
```

Clicking the button will open a dialog and start a session. `usecase` is what is communicated to the backend of the Relying Party when starting a session. See "starting a session"

### Options

Other options that can be passed to the custom element are:
- `start-url`: The URL to start a session. If not provided, it will default to `document.location.href`
- `text`: The text that is displayed on the button. Note that this *MUST* contain the words "NL Wallet". If not provided, it will default to `Login with NL Wallet`/`Inloggen met NL Wallet`
- `lang`: The language that is used for the text inside the dialog. Currently, only `en` (English) and `nl` (Dutch) are supported. If not provided, it will default to `nl`


### Events

The custom element provides two events: `success` and `failed`. They are triggered when the user closes the dialog after the session is completed successfully or not, respectively. The also return the session token and session type for the RP to decide what to do with failed sessions. The following example code shows how to subscribe to these events:
```javascript
const success = (e) => {
    if (e.detail && e.detail.length > 1) {
        const session_token = e.detail[0]
        const session_type = e.detail[1]
        if (session_type === "cross_device") {
            // ...
        }
    }
}

const failed = (e) => {
    // same as success
}

button.addEventListener("success", success, false)
button.addEventListener("failed", failed, false)
```

Generally, only `cross_device` sessions have to be dealt with, as same_device sessions will be moved to the background when opening the NL Wallet app.

### Styling

The style of the custom element is included in the JavaScript and will be injected in the shadow DOM. However, the button can be styled by using the following CSS selectors:
```css
nl-wallet-button::part(button) {
    /* The button itself */
}

nl-wallet-button::part(button-span) {
    /* The text inside the button */
}
```

It is not advised to style the dialog, as it is a part of the Wallet Web library and should be consistent across all RPs.

### Example

A full example could look like this:
```html
<!doctype html>
<html>
  <head>
    <title>NL Wallet example</title>
  </head>
  <style>
    nl-wallet-button::part(button) {
      background: blue;
      border: 0;
    }

    nl-wallet-button::part(button-span) {
      color: white;
      text-weight: bold;
    }
  </style>
  <body>
    <div id="app"></div>
      <h1>Login with NL Wallet</h1>
      <p>Click the button below to start a session</p>
      <nl-wallet-button
        text="Share with NL Wallet"
        usecase="my_usecase"
        start-url="https://example.com/my_usecase/start"
        lang="en"
      ></nl-wallet-button>
    </div>
  </body>

  <script>
    const wallet_button = document.getElementsByTagName("nl-wallet-button")
    const return_url = (e) => {
        if (e.detail && e.detail.length > 1) {
            const session_token = e.detail[0]
            const session_type = e.detail[1]

            // redirect to the return URL, but only on cross_device sessions
            if (session_type === "cross_device") {
                window.open(
                    "http://example.com/my_usecase/return/" + session_token
                )
            }
        }
    }

    button.addEventListener("success", return_url, false)
    button.addEventListener("failed", return_url, false)
  </script>
</html>
```

### Starting a session

In order to be able to start a session, a request has to be made to the Verification Server. This has to be done by a backend system of the RP, because it is insecure to leave this to frontend code. Therefore, RPs have to implement a backend endpoint that will start a session. The Wallet Web library will make a POST request to the `start-url` with the following body:
```json
{
    "usecase": "my_usecase"
}
```

And will expect a response with the following body:
```json
{
    "status_url": "https://example.com/disclosure/sessions/1234",
    "session_token": "1234"
}
```

This status URL should be the status URL of the Verification Server and is used by the Wallet Web library to poll the status of the session.

It is up to the RP to implement any session logic, if needed.

### CSP

In order to make use of the Wallet Web library in a secure way, it is advised to set a CSP header that allows the Wallet Web library to communicate with the Verification Server. The following elements should be included in the CSP header:
```
Content-Security-Policy
    script-src 'sha256-HASH_OF_WALLET_WEB_JS';
    style-src 'sha256-HASH_OF_STYLE_CSS';
    img-src data:;
    font-src data:;
```

The `img-src` and `font-src` are needed because the images (NL Wallet logo) and fonts are included as `data:` URIs in the library.

The hashes can be generated by running (assuming wallet web is built):

```sh
HASH_OF_WALLET_WEB_JS=sha256-$(cat dist/nl-wallet-web.iife.js | openssl sha256 -binary | openssl base64)
HASH_OF_STYLE_CSS=$(npm run --silent extract-style-hash)
```
