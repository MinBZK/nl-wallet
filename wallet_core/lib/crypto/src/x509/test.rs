use x509_parser::x509::X509Name;

use super::dn::DN_TYPE_ORGANIZATION_IDENTIFIER_OID;

pub fn assert_x509_common_name(x509name: &X509Name, common_name: &str) {
    itertools::assert_equal(x509name.iter_common_name().map(|a| a.as_str().unwrap()), [common_name]);
}

pub fn assert_x509_country_name(x509name: &X509Name, country_name: &str) {
    itertools::assert_equal(x509name.iter_country().map(|a| a.as_str().unwrap()), [country_name]);
}

pub fn assert_x509_organization_name(x509name: &X509Name, organization_name: &str) {
    itertools::assert_equal(
        x509name.iter_organization().map(|a| a.as_str().unwrap()),
        [organization_name],
    );
}

pub fn assert_x509_organization_identifier(x509name: &X509Name, organization_identifier: &str) {
    itertools::assert_equal(
        x509name
            .iter_by_oid(DN_TYPE_ORGANIZATION_IDENTIFIER_OID)
            .map(|a| a.as_str().unwrap()),
        [organization_identifier],
    );
}
