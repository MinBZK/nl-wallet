use coset::{iana, CoseMac0Builder, Header, HeaderBuilder};

use indexmap::IndexMap;
use p256::{PublicKey, SecretKey};
use wallet_common::keys::SecureEcdsaKey;

use crate::{
    errors::Result,
    iso::*,
    utils::{
        cose::{sign_cose, ClonePayload},
        crypto::dh_hmac_key,
        serialization::{cbor_serialize, TaggedBytes},
    },
};

impl DeviceSigned {
    pub async fn new_signature(private_key: &impl SecureEcdsaKey, challenge: &[u8]) -> Result<DeviceSigned> {
        let cose = sign_cose(challenge, Header::default(), private_key, false).await?;

        let device_signed = DeviceSigned {
            name_spaces: IndexMap::new().into(),
            device_auth: DeviceAuth::DeviceSignature(cose.into()),
        };

        Ok(device_signed)
    }

    #[allow(dead_code)] // TODO test this
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
