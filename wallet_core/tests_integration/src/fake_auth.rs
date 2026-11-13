use http_utils::reqwest::default_reqwest_client_builder;
use reqwest::Client;
use reqwest::header;
use reqwest::redirect::Policy;
use tracing::debug;
use url::Url;

pub fn extract_redirect_url(response: &reqwest::Response) -> Url {
    response
        .headers()
        .get(header::LOCATION)
        .expect("redirect should have location header")
        .to_str()
        .unwrap()
        .parse()
        .expect("failed to parse redirect url")
}

/// Create a reqwest client that does not follow redirects and return the redirect URL.
pub async fn setup_auth_redirect(authorization_url: Url) -> (Client, Url) {
    // TODO (PVW-5612): remove https_only(false) once the PID issuer runs on HTTPS.
    let client = default_reqwest_client_builder()
        .https_only(false)
        .redirect(Policy::none())
        .build()
        .unwrap();

    // The wallet's authorize URL (PAR already pushed) redirects to the issuer's consent page.
    let authorize_response = client.get(authorization_url).send().await.unwrap();
    let redirect_url = extract_redirect_url(&authorize_response);

    (client, redirect_url)
}

/// Drive the pid issuer's login ACF flow the way a browser would: follow the wallet's authorize URL
/// to the login page, submit bsn, and return the wallet-facing redirect (carrying the
/// issuer-minted code + the wallet's `state`) that [`Wallet::continue_issuance`] expects.
pub async fn fake_pid_auth(authorization_url: Url, bsn: String) -> Url {
    let (client, login_url) = setup_auth_redirect(authorization_url).await;

    debug!("login_url: {}", &login_url);

    // Login to the PID issuer using the BSN.
    let login_response = client.post(login_url).form(&[("bsn", bsn)]).send().await.unwrap();
    extract_redirect_url(&login_response)
}
