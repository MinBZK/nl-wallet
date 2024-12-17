use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use hex_literal::hex;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::EncodedPoint;
use p256::SecretKey;
use webpki::anchor_from_trusted_cert;
use webpki::types::CertificateDer;
use webpki::types::TrustAnchor;

pub struct Examples;

fn to_static_ref<T>(val: T) -> &'static mut T {
    Box::leak(Box::new(val))
}

pub const EXAMPLE_KEY_IDENTIFIER: &str = "example_static_device_key";

impl Examples {
    /// Returns the IACA trust anchor (Issuer Authority Certificate Authority).
    pub fn iaca_trust_anchors() -> &'static [TrustAnchor<'static>] {
        let bts = &hex!(
            "308201ce30820173a00302010202142ab4edd052b2582f4c6ad96186de70f4de5a3994300a06082a8648ce3d04030230233114301\
             206035504030c0b75746f7069612069616361310b3009060355040613025553301e170d3230313030313030303030305a170d3239\
             303932393030303030305a30233114301206035504030c0b75746f7069612069616361310b3009060355040613025553305930130\
             6072a8648ce3d020106082a8648ce3d030107034200042c3e103dbc07b25c5a770aeedfa5d8bd15417e3e676142461a7875e3b418\
             8a2221e6423599d1db19aaef66f923d394b61709549bcec2ea6ff60ec75268f2e094a38184308181301e0603551d1204173015811\
             36578616d706c65406578616d706c652e636f6d301c0603551d1f041530133011a00fa00d820b6578616d706c652e636f6d301d06\
             03551d0e0416041454fa2383a04c28e0d930792261c80c4881d2c00b300e0603551d0f0101ff04040302010630120603551d13010\
             1ff040830060101ff020100300a06082a8648ce3d0403020349003046022100ec897f0b8ae51028288955031f860069659b75989a\
             f7129fa609c24299a5c787022100d088d8741f5d05b360ef6e85023e90df1d31dd1e6701a88efe9a7103021f986c"
        );

        to_static_ref([anchor_from_trusted_cert(to_static_ref(CertificateDer::from(bts.as_slice()))).unwrap()])
    }

    /// CA cert for reader authentication
    pub fn reader_trust_anchors() -> &'static [TrustAnchor<'static>] {
        let bts = &hex!(
            "3082019030820137a003020102021430d747795405d564b7ac48be6f364ae2c774f2fc300a06082a8648ce3d04030230163114301\
             206035504030c0b72656164657220726f6f74301e170d3230313030313030303030305a170d3239303932393030303030305a3016\
             3114301206035504030c0b72656164657220726f6f743059301306072a8648ce3d020106082a8648ce3d030107034200043643293\
             832e0a480de592df0708fe25b6b923f6397ab39a8b1b7444593adb89c77b7e9c28cf48d6d187b43c9bf7b9c2c5c5ef22f329e44e7\
             a91b4745b7e2063aa3633061301c0603551d1f041530133011a00fa00d820b6578616d706c652e636f6d301d0603551d0e0416041\
             4cfb7a881baea5f32b6fb91cc29590c50dfac416e300e0603551d0f0101ff04040302010630120603551d130101ff040830060101\
             ff020100300a06082a8648ce3d0403020347003044022018ac84baf991a614fb25e76241857b7fd0579dfe8aed8ac7f1306754907\
             99930022077f46f00b4af3e014d253e0edcc9f146a75a6b1bdfe33e9fa72f30f0880d5237"
        );

        to_static_ref([anchor_from_trusted_cert(to_static_ref(CertificateDer::from(bts.as_slice()))).unwrap()])
    }

    /// Reader ephemeral private key, for deriving MAC key
    pub fn ephemeral_reader_key() -> SecretKey {
        ecdsa_keypair(
            "60e3392385041f51403051f2415531cb56dd3f999c71687013aac6768bc8187e",
            "e58deb8fdbe907f7dd5368245551a34796f7d2215c440c339bb0f7b67beccdfa",
            "de3b4b9e5f72dd9b58406ae3091434da48a6f9fd010d88fcb0958e2cebec947c",
        )
        .expect("ECDSA key parsing failed")
        .into()
    }

    /// Device private key corresponding to the public key in the MSO
    pub fn static_device_key() -> SigningKey {
        ecdsa_keypair(
            "96313d6c63e24e3372742bfdb1a33ba2c897dcd68ab8c753e4fbd48dca6b7f9a",
            "1fb3269edd418857de1b39a4e4a44b92fa484caa722c228288f01d0c03a2c3d6",
            "6ed542ad4783f0b18c833fadf2171273a35d969c581691ef704359cc7cf1e8c0",
        )
        .expect("ECDSA key parsing failed")
    }
}

// Functions for parsing ECDSA private/public key strings from ISO spec appendix D
fn ecdsa_keypair(x: &str, y: &str, d: &str) -> Result<SigningKey> {
    let sk = ecdsa_privkey(d)?;
    if *sk.verifying_key() != ecdsa_pubkey(x, y)? {
        bail!("keys don't match")
    }
    Ok(sk)
}

fn ecdsa_privkey(d: &str) -> Result<SigningKey> {
    let privkey = SigningKey::try_from(hex::decode(d)?.as_slice())?;
    Ok(privkey)
}

fn ecdsa_pubkey(x: &str, y: &str) -> Result<VerifyingKey> {
    VerifyingKey::from_encoded_point(&EncodedPoint::from_affine_coordinates(
        hex::decode(x)?.as_slice().into(),
        hex::decode(y)?.as_slice().into(),
        false,
    ))
    .context("failed to instantiate public key")
}
