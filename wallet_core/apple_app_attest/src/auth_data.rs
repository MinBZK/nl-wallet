use coset::CoseError;
use passkey_types::ctap2::{AuthenticatorData, Flags};

#[derive(Debug)]
pub struct AuthenticatorDataWithSource<const IS_TRUNCATED: bool>(Vec<u8>, AuthenticatorData);

impl<const IS_TRUNCATED: bool> TryFrom<Vec<u8>> for AuthenticatorDataWithSource<IS_TRUNCATED> {
    type Error = CoseError;

    fn try_from(mut value: Vec<u8>) -> Result<Self, Self::Error> {
        let auth_data = if IS_TRUNCATED {
            // The assertions Apple produces are not compliant with the specification,
            // as they have bit 6 set, yet do not include "attested credential data".
            // In order for the passkey-types crate to interpret this authenticator data
            // correctly, we temporarily zero out the bits that indicate any sort of
            // extension while parsing. We then restore the flags to their initial state,
            // as the hash over this data must be calculated over the original.
            let flags = value[32];
            // Unset any flag that signal extra data blocks after the main authenticator data.
            value[32] = flags & u8::from(!(Flags::AT | Flags::ED));

            let auth_data = AuthenticatorData::from_slice(&value)?;

            value[32] = flags;

            auth_data
        } else {
            AuthenticatorData::from_slice(&value)?
        };

        Ok(Self(value, auth_data))
    }
}

impl<const IS_TRUNCATED: bool> Clone for AuthenticatorDataWithSource<IS_TRUNCATED> {
    fn clone(&self) -> Self {
        // Guaranteed to succeed, since it was parsed once successfully before.
        Self::try_from(self.0.clone()).unwrap()
    }
}

impl<const IS_TRUNCATED: bool> AsRef<AuthenticatorData> for AuthenticatorDataWithSource<IS_TRUNCATED> {
    fn as_ref(&self) -> &AuthenticatorData {
        &self.1
    }
}

impl<const IS_TRUNCATED: bool> AuthenticatorDataWithSource<IS_TRUNCATED> {
    pub fn source(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "serialize")]
impl<const IS_TRUNCATED: bool> From<AuthenticatorDataWithSource<IS_TRUNCATED>> for Vec<u8> {
    fn from(value: AuthenticatorDataWithSource<IS_TRUNCATED>) -> Self {
        value.0
    }
}

#[cfg(feature = "serialize")]
impl<const IS_TRUNCATED: bool> From<AuthenticatorData> for AuthenticatorDataWithSource<IS_TRUNCATED> {
    fn from(value: AuthenticatorData) -> Self {
        let mut authenticator_data_bytes = value.to_vec();

        // Unfortunately Apple does not properly adhere to the Web Authentication specification,
        // as the authenticator data present in the generated assertions simply has their
        // "attested credential data" portion truncated, yet the "flags" still indicate that this
        // data is present. We reproduce that here by overriding that particular bit in the "flags".
        if IS_TRUNCATED {
            authenticator_data_bytes[32] |= u8::from(Flags::AT);
        }

        Self(authenticator_data_bytes, value)
    }
}
