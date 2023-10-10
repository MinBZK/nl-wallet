use cryptoki::{
    context::{CInitializeArgs, Pkcs11},
    mechanism::Mechanism,
    object::{Attribute, AttributeType},
    session::UserType,
    types::AuthPin,
};
use p256::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    pkcs8::{der, der::Decode, AssociatedOid},
    NistP256,
};
use sec1::{der::Encode, EcParameters};
use uuid::Uuid;

use wallet_common::utils::{random_bytes, sha256};
use wallet_provider::settings::Settings;

#[test]
fn it_works() {
    let settings = Settings::new().unwrap();

    let pkcs11_client = Pkcs11::new(settings.hsm.library_path).unwrap();
    pkcs11_client.initialize(CInitializeArgs::OsThreads).unwrap();

    let user_pin = AuthPin::new(settings.hsm.user_pin);

    let mut slots = dbg!(pkcs11_client.get_slots_with_initialized_token().unwrap());
    let slot = slots.remove(0);

    // open a session
    let session = pkcs11_client.open_rw_session(slot).unwrap();

    session.login(UserType::User, Some(&user_pin)).unwrap();

    let create_mechanism = Mechanism::EccKeyPairGen;
    let mechanism = Mechanism::Ecdsa;

    let mut oid = vec![];
    EcParameters::NamedCurve(NistP256::OID).encode_to_vec(&mut oid).unwrap();

    let pub_key_template = vec![
        Attribute::EcParams(oid),
        Attribute::Token(true), // ensure the public key is visible in Object Management. Do we want that?
    ];
    let priv_key_template = vec![
        Attribute::Token(true),
        Attribute::Private(true),
        Attribute::Sensitive(true),
        Attribute::Extractable(false),
        Attribute::Derive(false),
        // Attribute::Wrap(false), // settings this to false results in TemplateInconsistent err
        Attribute::Sign(true),
        Attribute::Label("wp_key_label".as_bytes().to_vec()),
        Attribute::Id(Uuid::new_v4().to_string().into()),
    ];

    // Generate key pair
    let (public, private) = session
        .generate_key_pair(&create_mechanism, &pub_key_template, &priv_key_template)
        .unwrap();

    let data = random_bytes(32);

    // sign something with it
    let signature = session.sign(&mechanism, private, &sha256(&data)).unwrap();

    // verify the signature by the HSM
    session.verify(&mechanism, public, &sha256(&data), &signature).unwrap();

    // verify the signature manually
    let attr = session
        .get_attributes(public, &[AttributeType::EcPoint])
        .unwrap()
        .remove(0);
    if let Attribute::EcPoint(ec_point) = attr {
        let octet_string = der::asn1::OctetString::from_der(&ec_point).unwrap();
        let public_key = VerifyingKey::from_sec1_bytes(octet_string.as_bytes()).unwrap();
        let sig = Signature::from_slice(&signature).unwrap();

        public_key.verify(&data, &sig).unwrap();
    } else {
        panic!("Expected attribute.");
    }

    // delete keys
    session.destroy_object(public).unwrap();
    session.destroy_object(private).unwrap();
}
