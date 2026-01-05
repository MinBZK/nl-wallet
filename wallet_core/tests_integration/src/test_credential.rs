use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::test_credential::TestCredential;
use attestation_data::test_credential::TestCredentials;
use attestation_types::status_claim::StatusClaim;
use pid_issuer::pid::constants::PID_ADDRESS_GROUP;
use pid_issuer::pid::constants::PID_BIRTH_DATE;
use pid_issuer::pid::constants::PID_BSN;
use pid_issuer::pid::constants::PID_FAMILY_NAME;
use pid_issuer::pid::constants::PID_GIVEN_NAME;
use pid_issuer::pid::constants::PID_RESIDENT_CITY;
use pid_issuer::pid::constants::PID_RESIDENT_COUNTRY;
use pid_issuer::pid::constants::PID_RESIDENT_HOUSE_NUMBER;
use pid_issuer::pid::constants::PID_RESIDENT_POSTAL_CODE;
use pid_issuer::pid::constants::PID_RESIDENT_STREET;
use utils::generator::mock::MockTimeGenerator;
use utils::vec_nonempty;

use crate::metadata::eudi_nl_pid_type_metadata_documents;

fn new_nl_pid<'a>(
    query_id: &str,
    query_claim_paths: impl IntoIterator<Item = impl IntoIterator<Item = &'a str>>,
) -> TestCredential {
    let (_, metadata_documents) = eudi_nl_pid_type_metadata_documents();

    TestCredential::new(
        PreviewableCredentialPayload::nl_pid_example(&MockTimeGenerator::default()),
        metadata_documents,
        query_id.parse().unwrap(),
        query_claim_paths,
        StatusClaim::new_mock(),
    )
}

pub fn new_nl_pid_all() -> TestCredential {
    new_nl_pid(
        "nl_pid_all",
        [[PID_GIVEN_NAME], [PID_FAMILY_NAME], [PID_BIRTH_DATE], [PID_BSN]],
    )
}

pub fn new_nl_pid_full_name() -> TestCredential {
    new_nl_pid("nl_pid_full_name", [[PID_GIVEN_NAME], [PID_FAMILY_NAME]])
}

pub fn new_nl_pid_given_name() -> TestCredential {
    new_nl_pid("nl_pid_given_name", [[PID_GIVEN_NAME]])
}

pub fn new_nl_pid_given_name_for_query_id(query_id: &str) -> TestCredential {
    new_nl_pid(query_id, [[PID_GIVEN_NAME]])
}

pub fn new_nl_pid_family_name() -> TestCredential {
    new_nl_pid("nl_pid_family_name", [[PID_FAMILY_NAME]])
}

pub fn new_nl_pid_address_all() -> TestCredential {
    new_nl_pid(
        "nl_pid_address_all",
        [
            [PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
            [PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
            [PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
            [PID_ADDRESS_GROUP, PID_RESIDENT_CITY],
            [PID_ADDRESS_GROUP, PID_RESIDENT_COUNTRY],
        ],
    )
}

pub fn new_nl_pid_address_minimal_address() -> TestCredential {
    new_nl_pid(
        "nl_pid_address_minimal_address",
        [
            [PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
            [PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
            [PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
        ],
    )
}

pub fn new_nl_pid_full_name_and_minimal_address() -> TestCredential {
    new_nl_pid(
        "nl_pid_full_name_and_minimal_address",
        [
            vec![PID_GIVEN_NAME],
            vec![PID_FAMILY_NAME],
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_STREET],
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_HOUSE_NUMBER],
            vec![PID_ADDRESS_GROUP, PID_RESIDENT_POSTAL_CODE],
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

pub fn nl_pid_full_name_and_minimal_address() -> TestCredentials {
    TestCredentials::new(vec_nonempty![new_nl_pid_full_name_and_minimal_address()])
}
