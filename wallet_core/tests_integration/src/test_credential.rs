use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::test_credential::TestCredential;
use attestation_data::test_credential::TestCredentials;
use attestation_types::status_claim::StatusClaim;
use dcql::CredentialQuery;
use dcql::Query;
use pid_issuer::pid::mock::mock_pid_example;
use pid_issuer::pid::mock::mock_pid_mdoc_example;
use pid_issuer::pid::rulebook::constants::PID_BSN;
use pid_issuer::pid::rulebook::constants::PID_FAMILY_NAME;
use pid_issuer::pid::rulebook::constants::PID_GIVEN_NAME;
use pid_issuer::pid::rulebook::constants::PID_MDOC_BIRTH_DATE;
use pid_issuer::pid::rulebook::constants::PID_MDOC_DOCTYPE;
use pid_issuer::pid::rulebook::constants::PID_MDOC_NAMESPACE;
use pid_issuer::pid::rulebook::constants::PID_MDOC_RESIDENT_CITY;
use pid_issuer::pid::rulebook::constants::PID_MDOC_RESIDENT_COUNTRY;
use pid_issuer::pid::rulebook::constants::PID_MDOC_RESIDENT_POSTAL_CODE;
use pid_issuer::pid::rulebook::constants::PID_MDOC_RESIDENT_STREET;
use pid_issuer::pid::rulebook::constants::PID_NL_SD_JWT_VCT;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_ADDRESS_GROUP;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_BIRTH_DATE;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_COUNTRY;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_HOUSE_NUMBER;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_LOCALITY;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_POSTAL_CODE;
use pid_issuer::pid::rulebook::constants::PID_SD_JWT_STREET_ADDRESS;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_nonempty;

use crate::metadata::eudi_nl_pid_type_metadata_documents;

fn new_nl_pid<'a>(
    query_id: &str,
    query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
    mdoc_query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
) -> TestCredential {
    let (_, type_metadata_documents) = eudi_nl_pid_type_metadata_documents();

    TestCredential::new(
        PreviewableCredentialPayload::example_with_attributes(
            PID_NL_SD_JWT_VCT,
            mock_pid_example(),
            &MockTimeGenerator::default(),
        ),
        PID_MDOC_DOCTYPE.to_owned(),
        mock_pid_mdoc_example(),
        Some(type_metadata_documents),
        query_id.parse().unwrap(),
        query_claim_paths,
        mdoc_query_claim_paths,
        StatusClaim::new_mock(),
    )
}

pub fn new_nl_pid_all() -> TestCredential {
    new_nl_pid(
        "nl_pid_all",
        [[PID_GIVEN_NAME], [PID_FAMILY_NAME], [PID_SD_JWT_BIRTH_DATE], [PID_BSN]],
        [
            [PID_MDOC_NAMESPACE, PID_GIVEN_NAME],
            [PID_MDOC_NAMESPACE, PID_FAMILY_NAME],
            [PID_MDOC_NAMESPACE, PID_MDOC_BIRTH_DATE],
            [PID_MDOC_NAMESPACE, PID_BSN],
        ],
    )
}

pub fn new_nl_pid_full_name() -> TestCredential {
    new_nl_pid(
        "nl_pid_full_name",
        [[PID_GIVEN_NAME], [PID_FAMILY_NAME]],
        [
            [PID_MDOC_NAMESPACE, PID_GIVEN_NAME],
            [PID_MDOC_NAMESPACE, PID_FAMILY_NAME],
        ],
    )
}

pub fn new_nl_pid_given_name() -> TestCredential {
    new_nl_pid(
        "nl_pid_given_name",
        [[PID_GIVEN_NAME]],
        [[PID_MDOC_NAMESPACE, PID_GIVEN_NAME]],
    )
}

pub fn new_nl_pid_given_name_for_query_id(query_id: &str) -> TestCredential {
    new_nl_pid(query_id, [[PID_GIVEN_NAME]], [[PID_MDOC_NAMESPACE, PID_GIVEN_NAME]])
}

pub fn new_nl_pid_family_name() -> TestCredential {
    new_nl_pid(
        "nl_pid_family_name",
        [[PID_FAMILY_NAME]],
        [[PID_MDOC_NAMESPACE, PID_FAMILY_NAME]],
    )
}

pub fn new_nl_pid_address_all() -> TestCredential {
    new_nl_pid(
        "nl_pid_address_all",
        [
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_STREET_ADDRESS],
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_HOUSE_NUMBER],
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_POSTAL_CODE],
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_LOCALITY],
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_COUNTRY],
        ],
        [
            [PID_MDOC_NAMESPACE, PID_MDOC_RESIDENT_STREET],
            [PID_MDOC_NAMESPACE, PID_MDOC_RESIDENT_POSTAL_CODE],
            [PID_MDOC_NAMESPACE, PID_MDOC_RESIDENT_CITY],
            [PID_MDOC_NAMESPACE, PID_MDOC_RESIDENT_COUNTRY],
        ],
    )
}

pub fn new_nl_pid_address_minimal_address() -> TestCredential {
    new_nl_pid(
        "nl_pid_address_minimal_address",
        [
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_STREET_ADDRESS],
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_HOUSE_NUMBER],
            [PID_SD_JWT_ADDRESS_GROUP, PID_SD_JWT_POSTAL_CODE],
        ],
        [
            [PID_MDOC_NAMESPACE, PID_MDOC_RESIDENT_STREET],
            [PID_MDOC_NAMESPACE, PID_MDOC_RESIDENT_POSTAL_CODE],
        ],
    )
}

pub fn nl_pid_credentials_all() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_all()])
}

pub fn nl_pid_credentials_full_name() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_full_name()])
}

pub fn nl_pid_credentials_given_name() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_given_name()])
}

pub fn nl_pid_credentials_given_name_for_query_id(query_id: &str) -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_given_name_for_query_id(query_id)])
}

pub fn nl_pid_credentials_family_name() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_family_name()])
}

pub fn nl_pid_address_credentials_all() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_address_all()])
}

pub fn nl_pid_address_minimal_address() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_address_minimal_address()])
}

pub fn new_mock_mdoc_pid_example() -> Query {
    Query::new_mock_single(CredentialQuery::new_mock_mdoc(
        "mdoc_pid_example",
        PID_MDOC_DOCTYPE,
        PID_MDOC_NAMESPACE,
        &["bsn", "given_name", "family_name"],
    ))
}
