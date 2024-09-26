use coset::CoseError;
use passkey_types::ctap2::AuthenticatorData;

#[derive(Debug)]
pub struct AuthenticatorDataWithSource(Vec<u8>, AuthenticatorData);

impl TryFrom<Vec<u8>> for AuthenticatorDataWithSource {
    type Error = CoseError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let auth_data = AuthenticatorData::from_slice(&value)?;

        Ok(Self(value, auth_data))
    }
}

impl AsRef<AuthenticatorData> for AuthenticatorDataWithSource {
    fn as_ref(&self) -> &AuthenticatorData {
        &self.1
    }
}

impl AuthenticatorDataWithSource {
    pub fn source(&self) -> &[u8] {
        &self.0
    }
}
