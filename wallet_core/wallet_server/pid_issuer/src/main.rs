use std::convert::Infallible;
use std::sync::Arc;

use anyhow::Result;
use health_checkers::hsm::HsmChecker;
use hsm::service::Pkcs11Hsm;
use pid_issuer::pid::attributes::Person;
use pid_issuer::pid::attributes::Persons;
use pid_issuer::pid::auth_code_flow::LoginAuthorizationCodeFlow;
use pid_issuer::pid::login_page::LoginPage;
use pid_issuer::pid::login_page::build_login_csp;
use pid_issuer::pid::rulebook::issuer_metadata::PidIssuerMetadata;
use pid_issuer::server;
use pid_issuer::settings::PidIssuerSettings;
use server_utils::keys::SecretKeyVariant;
use server_utils::server::wallet_server_main;

#[tokio::main]
async fn main() -> Result<()> {
    wallet_server_main("pid_issuer.toml", "pid_issuer", main_impl).await
}

async fn main_impl(settings: PidIssuerSettings) -> Result<()> {
    let serve_status_lists = settings.authorizing_issuer_settings.issuer_settings.status_lists.serve;

    // Create HSM
    let hsm = settings
        .authorizing_issuer_settings
        .issuer_settings
        .server_settings
        .hsm
        .clone()
        .map(Pkcs11Hsm::from_settings)
        .transpose()?;
    let hsm_checker = hsm.as_ref().map(HsmChecker::new);

    // Load persons and login subjects, use all persons if login subjects is empty
    let persons: Vec<Person> =
        serde_json::from_str(&tokio::fs::read_to_string(&settings.person_attributes_path).await?)?;
    let mut login_subjects = settings.login_subjects;
    if login_subjects.is_empty() {
        // Use Vec<Person> to keep order of persons
        login_subjects = persons.iter().map(|p| p.bsn.clone()).collect::<Vec<_>>();
    }
    let persons = Persons::try_from(persons)?;

    // The login page sets its own CSP, whose `form-action` must allow the wallet redirect target.
    // Build it while the redirect URIs are in scope and leak it for the 'static lifetime the
    // response middleware requires (it lives for the process, like the server).
    let csp: &'static str =
        Box::leak(build_login_csp(&settings.authorizing_issuer_settings.wallet_redirect_uris).into_boxed_str());
    let login_page = LoginPage::new(
        login_subjects.into_iter().flat_map(|bsn| persons.as_ref().get(&bsn)),
        settings.login_custom_bsn,
        csp,
    );

    let recovery_code_secret_key = SecretKeyVariant::from_settings(settings.recovery_code, hsm.clone())?;
    let issuer_metadata = PidIssuerMetadata {
        issuing_country: settings.issuing_country,
        issuing_authority: settings.issuing_authority,
    };

    let public_url = settings
        .authorizing_issuer_settings
        .issuer_settings
        .public_url
        .as_base_url()
        .clone();
    let login_url = public_url.join(login_page.path());

    let (issuer, database_checkers, _, server_settings) = settings
        .authorizing_issuer_settings
        .into_authorizing_issuer(hsm, |store_connection| {
            Ok::<_, Infallible>(LoginAuthorizationCodeFlow::new(
                recovery_code_secret_key,
                issuer_metadata,
                store_connection,
                login_url,
                persons,
            ))
        })
        .await?;

    let authorizing_issuer = Arc::new(issuer);

    let health_checkers = health_checkers::boxed(hsm_checker)
        .into_iter()
        .chain(database_checkers.into_iter().map(|checker| Box::new(checker) as Box<_>));

    // This will block until the server shuts down.
    server::serve(
        authorizing_issuer,
        server_settings,
        serve_status_lists,
        health_checkers,
        login_page,
    )
    .await
}
