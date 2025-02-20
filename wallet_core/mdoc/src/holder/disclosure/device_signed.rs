use coset::iana;
use coset::CoseMac0Builder;
use coset::Header;
use coset::HeaderBuilder;

use indexmap::IndexMap;
use p256::PublicKey;
use p256::SecretKey;

use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;

use crate::errors::Result;
use crate::iso::*;
use crate::utils::cose::sign_coses;
use crate::utils::cose::ClonePayload;
use crate::utils::crypto::dh_hmac_key;
use crate::utils::serialization::cbor_serialize;
use crate::utils::serialization::TaggedBytes;

impl DeviceSigned {
    pub async fn new_signatures<K, KF>(
        keys_and_challenges: Vec<(K, &[u8])>,
        key_factory: &KF,
    ) -> Result<(Vec<DeviceSigned>, Vec<K>)>
    where
        K: CredentialEcdsaKey,
        KF: KeyFactory<Key = K>,
    {
        let (coses, keys) = sign_coses(keys_and_challenges, key_factory, Header::default(), false).await?;

        let signed = coses
            .into_iter()
            .map(|cose| DeviceSigned {
                name_spaces: IndexMap::new().into(),
                device_auth: DeviceAuth::DeviceSignature(cose.into()),
            })
            .collect();

        Ok((signed, keys))
    }

    pub fn new_mac(
        private_key: &SecretKey,
        reader_pub_key: &PublicKey,
        session_transcript: &SessionTranscript,
        device_auth: &DeviceAuthenticationBytes,
    ) -> Result<DeviceSigned> {
        let key = dh_hmac_key(
            private_key,
            reader_pub_key,
            &cbor_serialize(&TaggedBytes(session_transcript))?,
            "EMacKey",
            32,
        )?;

        let cose = CoseMac0Builder::new()
            .payload(cbor_serialize(device_auth)?)
            .protected(HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build())
            .create_tag(&[], |data| ring::hmac::sign(&key, data).as_ref().into())
            .build()
            .clone_without_payload();

        let device_signed = DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceMac(cose.into()),
        };
        Ok(device_signed)
    }
}

#[cfg(test)]
mod tests {
    use p256::SecretKey;

    use wallet_common::keys::examples::Examples;

    use crate::examples::Example;
    use crate::examples::IsoCertTimeGenerator;
    use crate::holder::Mdoc;
    use crate::DeviceAuthenticationBytes;
    use crate::DeviceSigned;
    use crate::Document;

    #[test]
    fn test_mac_device_signed() {
        let (mdoc, ca) = Mdoc::new_example_mock();
        let eph_reader_key = Examples::ephemeral_reader_key();
        let session_transcript = DeviceAuthenticationBytes::example().0 .0.session_transcript;

        // We grab the private key directly from the `Examples` instead of obtaining a `LocalEcdsaKey` from `mdoc`,
        // because we need to access it directly in this test to convert it to a `SecretKey`.
        let secret_key = SecretKey::from(Examples::static_device_key().as_nonzero_scalar());

        let mac_device_signed = DeviceSigned::new_mac(
            &secret_key,
            &eph_reader_key.public_key(),
            &session_transcript,
            &DeviceAuthenticationBytes::example(),
        )
        .unwrap();

        let document = Document {
            doc_type: mdoc.doc_type().clone(),
            issuer_signed: mdoc.issuer_signed.clone(),
            device_signed: mac_device_signed,
            errors: None,
        };

        document
            .verify(
                Some(&eph_reader_key),
                &session_transcript,
                &IsoCertTimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap();
    }
}
