use attestation_types::claim_path::ClaimPath;
use crypto::CredentialEcdsaKey;
use crypto::wscd::DisclosureWscd;
use utils::vec_at_least::VecNonEmpty;

use crate::errors::Result;
use crate::iso::disclosure::IssuerSigned;
use crate::iso::mdocs::DeviceKey;
use crate::iso::mdocs::DocType;

use super::super::Mdoc;
use super::MissingAttributesError;

#[derive(Debug, Clone)]
pub struct DisclosureMdoc {
    pub doc_type: DocType,
    pub issuer_signed: IssuerSigned,
    pub(super) private_key_id: String,
    pub(super) device_key: DeviceKey,
}

impl DisclosureMdoc {
    pub fn try_new<'a>(
        mdoc: Mdoc,
        claim_paths: impl IntoIterator<Item = &'a VecNonEmpty<ClaimPath>>,
    ) -> std::result::Result<Self, MissingAttributesError> {
        let issuer_signed = mdoc.issuer_signed.into_attribute_subset(claim_paths)?;

        let disclosure_mdoc = Self {
            doc_type: mdoc.mso.doc_type,
            issuer_signed,
            private_key_id: mdoc.private_key_id,
            device_key: mdoc.mso.device_key_info.device_key,
        };

        Ok(disclosure_mdoc)
    }

    pub(super) fn credential_key<K, W>(&self, wscd: &W) -> Result<K>
    where
        K: CredentialEcdsaKey,
        W: DisclosureWscd<Key = K>,
    {
        let public_key = (&self.device_key).try_into()?;
        let key = wscd.new_key(&self.private_key_id, public_key);

        Ok(key)
    }
}

#[cfg(any(test, feature = "mock_example_constructors"))]
mod examples {

    use std::sync::LazyLock;

    use futures::FutureExt;

    use attestation_types::claim_path::ClaimPath;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::generate::Ca;
    use utils::vec_at_least::VecNonEmpty;

    use crate::holder::Mdoc;
    use crate::test::data::PID;

    use super::DisclosureMdoc;

    static PID_EXAMPLE_CLAIM_PATHS: LazyLock<Vec<VecNonEmpty<ClaimPath>>> = LazyLock::new(|| {
        ["bsn", "given_name", "family_name"]
            .iter()
            .map(|attr| {
                vec![
                    ClaimPath::SelectByKey(PID.to_string()),
                    ClaimPath::SelectByKey(attr.to_string()),
                ]
                .try_into()
                .unwrap()
            })
            .collect()
    });

    impl DisclosureMdoc {
        /// Create a mock [`DisclosureMdoc`] with all the attributes from the PID example.
        pub fn new_mock_with_ca_and_key(ca: &Ca, device_key: &MockRemoteEcdsaKey) -> Self {
            let mdoc = Mdoc::new_mock_with_ca_and_key(ca, device_key).now_or_never().unwrap();

            Self::try_new(mdoc, PID_EXAMPLE_CLAIM_PATHS.iter()).unwrap()
        }
    }
}
