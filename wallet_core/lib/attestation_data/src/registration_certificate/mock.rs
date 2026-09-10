use cose::wrprc_cwt::SignedWrprcCwt;
use crypto::server_keys::generate::Ca;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use dcql::ClaimsSelection;
use dcql::CredentialQuery;
use dcql::Query;
use futures::FutureExt;
use jwt::SignedJwt;
use jwt::jades_b_b::JadesbbHeader;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use token_status_list::status_list::StatusList;
use token_status_list::status_list::StatusType;
use token_status_list::status_list_token::StatusListToken;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::client::StatusListClientError;
use url::Url;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use super::Credential;
use crate::x509::RelyingParty;

pub const ANNEX_C_EXAMPLE: &str = include_str!("../../examples/spec/registration_certificate_annex_c.json");
pub const STATUS_LIST_URI: &str = "https://example.com/statuslists/1";

#[derive(Serialize)]
#[serde(transparent)]
pub struct RegistrationCertificateFixture(pub Value);

impl jwt::JwtTyp for RegistrationCertificateFixture {
    const TYP: &'static str = jwt::jades_b_b::JADES_B_B_JWT_TYP;
}

#[derive(Debug, Clone)]
pub struct StaticStatusListClient(pub StatusListToken);

impl StatusListClient for StaticStatusListClient {
    async fn fetch(&self, _url: Url) -> Result<StatusListToken, StatusListClientError> {
        Ok(self.0.clone())
    }
}

fn registration_certificate_credentials(query: Query) -> Vec<Credential> {
    query
        .credentials
        .into_iter()
        .map(|credential| {
            let CredentialQuery {
                format,
                claims_selection,
                ..
            } = credential;
            let claim = match claims_selection {
                ClaimsSelection::NoSelectivelyDisclosable => None,
                ClaimsSelection::Combinations { claims, .. } | ClaimsSelection::All { claims } => {
                    Some(claims.into_inner())
                }
            };

            Credential { format, claim }
        })
        .collect()
}

pub fn registration_certificate_payload(
    access_certificate: &BorrowingCertificate,
    query: Query,
) -> RegistrationCertificateFixture {
    let relying_party = RelyingParty::try_from(access_certificate.to_distinguished_name().unwrap()).unwrap();
    let (subject, subject_fields) = match relying_party {
        RelyingParty::LegalPerson {
            organization_identifier,
            organization_name,
            ..
        } => (organization_identifier, json!({ "sub_ln": organization_name })),
        RelyingParty::NaturalPerson {
            serial_number,
            given_name,
            surname,
            ..
        } => (serial_number, json!({ "sub_gn": given_name, "sub_fn": surname })),
    };
    let mut payload = json!({
        "id": "mock-registration-certificate",
        "name": "Mock verifier",
        "sub": subject,
        "country": "NL",
        "registry_uri": "https://example.com/register",
        "support_uri": "support@example.com",
        "srv_description": [[{ "lang": "en", "value": "Mock verification service" }]],
        "supervisory_authority": {},
        "entitlements": ["https://uri.etsi.org/19475/Entitlement/Service_Provider"],
        "credentials": registration_certificate_credentials(query),
        "purpose": [{ "lang": "en", "value": "Testing" }],
        "iat": TimeGenerator.generate().timestamp(),
        "status": {
            "idx": "0",
            "uri": STATUS_LIST_URI
        },
        "policy_id": ["0.4.0.19475.3.1"],
        "certificate_policy": "https://example.com/policy"
    });
    payload.as_object_mut().unwrap().extend(
        subject_fields
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );

    RegistrationCertificateFixture(payload)
}

#[derive(Debug, Clone)]
pub struct MockRegistrationCertificate {
    pub certificate: Vec<u8>,
    pub trust_anchors: TrustAnchors,
    pub status_list_client: StaticStatusListClient,
}

impl MockRegistrationCertificate {
    pub fn new(access_certificate: &BorrowingCertificate, query: Query) -> Self {
        let authority = MockRegistrationCertificateAuthority::new();
        let certificate = authority.issue_jwt(access_certificate, query);

        Self {
            certificate,
            trust_anchors: authority.trust_anchors,
            status_list_client: authority.status_list_client,
        }
    }
}

pub struct MockRegistrationCertificateAuthority {
    ca: Ca,
    pub trust_anchors: TrustAnchors,
    pub status_list_client: StaticStatusListClient,
}

impl Default for MockRegistrationCertificateAuthority {
    fn default() -> Self {
        Self::new()
    }
}

impl MockRegistrationCertificateAuthority {
    pub fn new() -> Self {
        Self::new_with_status(StatusType::Valid)
    }

    pub fn new_with_status(status: StatusType) -> Self {
        let ca = Ca::generate_mock();
        let trust_anchors = TrustAnchors::from(&ca);
        let status_list_key_pair = ca.generate_issuer_status_list_mock().unwrap();
        let mut status_list = StatusList::new(10);
        if status != StatusType::Valid {
            assert_eq!(status_list.insert(0, status), None);
        }
        let status_list = StatusListToken::builder(STATUS_LIST_URI.parse().unwrap(), status_list.pack())
            .sign(&status_list_key_pair)
            .now_or_never()
            .unwrap()
            .unwrap();

        Self {
            ca,
            trust_anchors,
            status_list_client: StaticStatusListClient(status_list),
        }
    }

    pub fn issue_jwt(&self, access_certificate: &BorrowingCertificate, query: Query) -> Vec<u8> {
        let signing_key_pair = self.ca.generate_issuer_mock().unwrap();
        SignedJwt::<_, JadesbbHeader>::sign_with_iat(
            &registration_certificate_payload(access_certificate, query),
            &signing_key_pair,
            &TimeGenerator,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
        .to_string()
        .into_bytes()
    }

    pub fn issue_cwt(&self, access_certificate: &BorrowingCertificate, query: Query) -> Vec<u8> {
        let signing_key_pair = self.ca.generate_issuer_mock().unwrap();
        SignedWrprcCwt::sign_with_certificate(
            &registration_certificate_payload(access_certificate, query),
            &signing_key_pair,
            &TimeGenerator,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
        .to_vec()
        .unwrap()
    }
}
