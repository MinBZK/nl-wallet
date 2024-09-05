use std::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
};

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use webpki::{Error, TrustAnchor};

/// A version of [`TrustAnchor`] that can more easily be used as a field
/// in another struct, as it does not require a liftetime annotation.
///
/// Can be converted from a reference to a [`TrustAnchor`] or a byte-slice
/// reference `&[u8]` using the `From<>` trait. Conversely a [`TrustAnchor`]
/// may be created from a reference to [`OwnedTrustAnchor`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OwnedTrustAnchor {
    subject: Vec<u8>,
    spki: Vec<u8>,
    name_constraints: Option<Vec<u8>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct DerTrustAnchor {
    pub owned_trust_anchor: OwnedTrustAnchor,
    pub der_bytes: Vec<u8>,
}

impl Debug for DerTrustAnchor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.der_bytes.fmt(f)
    }
}

impl Hash for DerTrustAnchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.der_bytes.hash(state)
    }
}

impl DerTrustAnchor {
    pub fn from_der(der_bytes: Vec<u8>) -> Result<Self, Error> {
        let der = Self {
            owned_trust_anchor: TrustAnchor::try_from_cert_der(&der_bytes).map(|anchor| (&anchor).into())?,
            der_bytes,
        };
        Ok(der)
    }
}

impl From<&TrustAnchor<'_>> for OwnedTrustAnchor {
    fn from(value: &TrustAnchor) -> Self {
        OwnedTrustAnchor {
            subject: value.subject.to_vec(),
            spki: value.spki.to_vec(),
            name_constraints: value.name_constraints.map(|nc| nc.to_vec()),
        }
    }
}

impl<'a> From<&'a OwnedTrustAnchor> for TrustAnchor<'a> {
    fn from(value: &'a OwnedTrustAnchor) -> Self {
        TrustAnchor {
            subject: &value.subject,
            spki: &value.spki,
            name_constraints: value.name_constraints.as_deref(),
        }
    }
}

impl Serialize for DerTrustAnchor {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        BASE64_STANDARD.encode(&self.der_bytes).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DerTrustAnchor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let der_bytes = BASE64_STANDARD
            .decode(String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)?;
        DerTrustAnchor::from_der(der_bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::trust_anchor::DerTrustAnchor;

    #[test]
    fn der_trust_anchor_serialization() {
        let anchor_str = "MIIBkzCCATqgAwIBAgIUOCjkeBboSUVO3A+Wq8Xb4Ize3twwCgYIKoZIzj0EAwIwGTEXMBUGA1UEAwwOY2EuZXhhbXBs\
            ZS5jb20wHhcNMjMxMTE3MDc1OTQzWhcNMjQxMTE2MDc1OTQzWjAZMRcwFQYDVQQDDA5jYS5leGFtcGxlLmNvbTBZMBMGByqGSM49AgEGCC\
            qGSM49AwEHA0IABMwoWnLasOGW6ogQ0TeojJTOAQirhLkxX0rqWGXe97sb6LrfsUGx5URdzNhXO8REBZyhszEH+xrYEX5hBPGvXnOjYDBe\
            MB0GA1UdDgQWBBS6toHYF2P6gnKEnMjYuXRvqwFLmTAfBgNVHSMEGDAWgBS6toHYF2P6gnKEnMjYuXRvqwFLmTAPBgNVHRMBAf8EBTADAQ\
            H/MAsGA1UdDwQEAwIBBjAKBggqhkjOPQQDAgNHADBEAiB16lDCCRPtST/h3mYM86V7FhodF47j0OZWY57jmDxstQIgQHt8XU2CYYCSSt42\
            nw4CJrY9QCwosFay0VSMh9nqUMA=";

        let json_anchor_str = format!("\"{}\"", anchor_str);

        let deserialized: DerTrustAnchor = serde_json::from_str(&json_anchor_str).unwrap();
        let serialized_anchor = serde_json::to_string(&deserialized).unwrap();

        assert_eq!(json_anchor_str, serialized_anchor.as_str());
    }
}
