//! AES-SIV-CMAC-512, the deterministic authenticated encryption mode of [RFC 5297], built out of
//! AES-CTR and AES-CMAC.
//!
//! This implementation (unlike e.g. RustCrypto's `aes-siv` implementation) supports using
//! an HSM for securing the key material, assuming that HSM supports AES-CTR and AES-CMAC.
//! See the [`AesSivBackend`] trait, and the functions [`aes_siv_encrypt()`] and
//! [`aes_siv_decrypt()`].
//!
//! # Determinism
//!
//! SIV takes no random nonce or IV. The initialization vector for CTR mode is instead *synthesized*
//! from the plaintext, and doubles as the integrity tag. Encryption is therefore a pure function of
//! the key and the plaintext: encrypting the same plaintext twice under the same key returns
//! byte-identical ciphertext.
//!
//! This is a deliberate property of the mode and not a defect, but it does have an important
//! consequence that callers have to accept:
//!
//! > Anyone who sees two ciphertexts learns whether the plaintexts behind them are equal.
//!
//! Nothing beyond that equality leaks.
//!
//! # When to (not) use this
//!
//! Use this ONLY when the ciphertext needs to be deterministic. For example, when storing a set of
//! encrypted identifiers that needs to be searchable (e.g. an index on a corresponding database
//! table).
//!
//! In all other cases, consider a conventional, probabilistic AEAD such as AES-GCM.
//!
//! # The construction, and the names the RFC gives its parts
//!
//! The code below uses the RFC's one-letter names throughout, so they are introduced here once.
//! The key is `K`, of 512 bits, split into halves `K1` and `K2`; the plaintext is `P`.
//! Encryption is then two steps:
//!
//! 1. `V = S2V(K1, P)`.
//! 2. `C = AES-CTR(K2, Q, P)`, where `Q` is `V` with two bits cleared.
//!
//! The output is `Z = V || C`: the 16-byte `V` in front, the ciphertext `C` behind it.
//!
//! S2V ("string to vector") is the part that is specific to this mode, and is what the local
//! `s2v()` implements. It is a pseudorandom function, built out of AES-CMAC, that compresses a key
//! and a *list* of input strings into one 128-bit value. The RFC feeds it the associated data
//! followed by the plaintext; with no associated data, as here, that list holds nothing but `P`,
//! and S2V reduces to two AES-CMAC calls. Its output `V` then does double duty: it is the integrity
//! tag, and it is (after masking) the counter block `Q` that CTR mode starts from — hence
//! *synthetic* IV.
//!
//! Decryption runs the same two steps backwards: split `V` off the front of `Z`, decrypt `C` with
//! it, then recompute S2V over the recovered plaintext and check that it reproduces the `V` that
//! arrived in the ciphertext. If it does, the ciphertext was produced by someone holding `K`, and
//! neither `V` nor `C` has been altered since.
//!
//! [RFC 5297]: https://datatracker.ietf.org/doc/html/rfc5297
//!
//! # Security
//!
//! - AES-SIV is a handful of operations arranged around two cryptographic primitives, AES-CMAC and AES-CTR. Both of
//!   those, and all of the key material they consume, live behind [`AesSivBackend`], i.e. in the HSM. Neither `K1` nor
//!   `K2` is ever visible to the code in this module.
//! - Timing side channels are avoided by not branching over secret values. In particular, during decryption the
//!   recomputed integrity tag is compared in constant time against the one from the ciphertext. Secret-dependent memory
//!   access is also avoided in order to prevent cache or memory access side channels.
//! - Beyond timing, plaintext is kept out of memory for longer than it needs to be: the intermediate `T` in [`s2v()`],
//!   and the plaintext buffers in both directions are held in `Zeroizing`, so they are cleared on the error paths as
//!   well as the success path. On decryption in particular, no plaintext byte leaves the function unless the tag check
//!   passed.
//!
//! Two things this module cannot enforce, and which are therefore the caller's responsibility:
//! `K1` and `K2` must be distinct (see [`AesSivKey::try_new()`], whose check is best-effort only),
//! and the backend must implement the two primitives faithfully. In particular, it must use the
//! counter block exactly as given, as [`AesSivBackend::aes_ctr()`] spells out.
//!
//! # Limitations
//!
//! This implementation is kept as small as possible, in the sense that it only implements those
//! parts of the RFC that are currently used in this codebase. As such, this implementation has
//! the following limitations.
//!
//!  - Associated data (AD) is not supported.
//!  - AES-SIV treats plaintexts smaller than 128 bits (16 bytes) differently from larger plaintexts. This
//!    implementation supports only plaintexts whose length equals or exceeds 128 bits.

use derive_more::Debug;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

#[derive(Debug, thiserror::Error)]
pub enum AesSivError {
    #[error("plaintext too short: must be at least 16 bytes")]
    PlaintextTooShort,
    #[error("ciphertext too short: must be at least 32 bytes")]
    CiphertextTooShort,
    #[error("AES-SIV backend error: {0}")]
    BackendError(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Tampering, a wrong CMAC key and a wrong CTR key all end up here, and the message is
    /// deliberately vague about which: a caller that can tell those apart could learn something
    /// about the key it is holding.
    #[error("decryption failed")]
    AuthenticationFailed,
}

#[derive(Debug, thiserror::Error)]
#[error("AES-SIV keys must be distinct, but were equal")]
pub struct AesSivKeysEqualError;

/// An AES-SIV key, consisting of a key for AES-CMAC and another key for AES-CTR.
///
/// In practice, K1 and K2 are always going to be the same type:
/// - When using HSMs, one refers to keys of any kind using a string or some sort of handle;
/// - When implementing `AesSivBackend` directly on in-memory keys, then both key fields would be of type `[u8; 32]`.
///
/// Nevertheless, here they are modeled as keys of distinct types for type safety. The only
/// public way of constructing instances of this type is through `AesSivKey::<K, K>::try_new()`.
#[derive(Debug)]
#[debug("<AesSivKey>")]
pub struct AesSivKey<K1, K2> {
    mac_key: K1,
    encryption_key: K2,
}

impl<K: Eq> AesSivKey<K, K> {
    /// Construct a new AES-SIV key.
    ///
    /// NOTE: This constructor checks that its two arguments are not equal, because handing AES-SIV two
    /// identical keys would break security. This equality check is however no more than a best effort,
    /// because in the case of a HSM, one can always construct two unequal references to a single key.
    /// This cannot be prevented here. It is the callers responsibility to not do this!
    pub fn try_new(mac_key: K, encryption_key: K) -> Result<Self, AesSivKeysEqualError> {
        if mac_key == encryption_key {
            return Err(AesSivKeysEqualError);
        }

        Ok(Self {
            mac_key,
            encryption_key,
        })
    }
}

/// Types that can perform AES-CTR-256 and AES-CMAC-256.
///
/// For types implementing this trait, AES-SIV-CMAC-512 is available using
/// [`aes_siv_encrypt()`] and [`aes_siv_decrypt()`].
///
/// AES-SIV takes a 512-bit key, which [RFC 5297, section 2.6] splits into two equal halves, and the
/// two associated key types here are those halves: K1 is a [`MacKey`](AesSivBackend::MacKey) and
/// keys [`aes_cmac()`](AesSivBackend::aes_cmac), while K2 is an
/// [`EncryptionKey`](AesSivBackend::EncryptionKey) and keys [`aes_ctr()`](AesSivBackend::aes_ctr).
///
/// [RFC 5297, section 2.6]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.6
pub trait AesSivBackend {
    type Error: std::error::Error + Send + Sync + 'static;

    type MacKey;
    type EncryptionKey;

    /// AES-CMAC over the whole of `input`, keyed with K1, returning the 128-bit tag.
    ///
    /// Every call has to be a complete MAC in itself: no state may carry over from one call to the
    /// next.
    async fn aes_cmac(
        &self,
        key: &Self::MacKey,
        input: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<[u8; 16], Self::Error>;

    /// AES-CTR over `input`, keyed with K2, starting from `counter_block`.
    ///
    /// Note that AES-CTR is symmetric: this one function serves for both encryption and decryption.
    ///
    /// One requirement on the implementation, which is a silent interoperability failure rather
    /// than an error when it is not met: `counter_block` must be used exactly as given. A backend
    /// that generates its own IV, or prepends one to its output, breaks the construction, since SIV
    /// derives the counter from the plaintext precisely so that no separate IV exists.
    ///
    /// [RFC 5297, section 2.5]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.5
    async fn aes_ctr(
        &self,
        key: &Self::EncryptionKey,
        counter_block: [u8; 16],
        input: impl AsRef<[u8]> + Send + 'static,
    ) -> Result<Vec<u8>, Self::Error>;
}

/// AES-SIV-CMAC-512 encryption as defined in [RFC 5297, section 2.6].
/// For decryption, see [`aes_siv_decrypt`].
///
/// The return value is `Z = V || C`: the 16-byte tag, followed by a ciphertext of exactly the
/// length of the plaintext. Ciphertext expansion is thus 16 bytes, whatever the input.
///
/// Deriving `V` requires the entire plaintext before the first byte of ciphertext can be produced,
/// so this is inherently two-pass and cannot be turned into a streaming API. It costs three round
/// trips to the backend: the two [`aes_cmac()`](AesSivBackend::aes_cmac) calls that make up S2V,
/// then one [`aes_ctr()`](AesSivBackend::aes_ctr) call.
///
/// # Limitations
///
/// This implements a subset of the RFC:
///
/// - No associated data is supported.
/// - The plaintext must be at least 128 bits, i.e. 16 bytes; shorter ones are rejected with
///   [`AesSivError::PlaintextTooShort`].
///
/// [RFC 5297, section 2.6]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.6
pub async fn aes_siv_encrypt<K: AesSivBackend>(
    backend: &K,
    key: &AesSivKey<K::MacKey, K::EncryptionKey>,
    plaintext: Vec<u8>,
) -> Result<Vec<u8>, AesSivError> {
    if plaintext.len() < 16 {
        return Err(AesSivError::PlaintextTooShort);
    }

    // Forget our `plaintext` copy as soon as possible.
    let plaintext = Zeroizing::new(plaintext);

    // V = S2V(K1, AD1, ..., ADn, P), with n = 0.
    // This acts as the integrity tag.
    let v = s2v(backend, &key.mac_key, &plaintext).await?;

    // Q = V bitand (1^64 || 0^1 || 1^31 || 0^1 || 1^31)
    // The AES-CTR counter block.
    let q = ctr_iv(v);

    // C = CTR(K2, Q, P), and the return value is Z = V || C, so V goes in front and only the
    // plaintext following it is run through the keystream.
    let ciphertext = backend
        .aes_ctr(&key.encryption_key, q, plaintext)
        .await
        .map_err(|e| AesSivError::BackendError(e.into()))?;

    Ok([&v, ciphertext.as_slice()].concat())
}

/// AES-SIV-CMAC-512 decryption from [RFC 5297, section 2.7], the inverse of [`aes_siv_encrypt`].
///
/// Note that this decrypts before it authenticates, which is the opposite of the usual advice. SIV
/// leaves no choice: V is at once the integrity tag and the CTR counter, so the plaintext has to be
/// recovered before the tag over it can be recomputed and compared. This is exactly what
/// [RFC 5297, section 2.7] prescribes, and it is safe here because the plaintext never leaves this
/// function unless that comparison succeeds.
///
/// # Limitations
///
/// This implements a subset of the RFC:
///
/// - no associated data is supported.
/// - The ciphertext must be at least 32 bytes, i.e. 256 bits. Shorter ciphertexts are rejected with
///   [`AesSivError::CiphertextTooShort`].
///
/// [RFC 5297, section 2.7]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.7
pub async fn aes_siv_decrypt<K: AesSivBackend>(
    backend: &K,
    key: &AesSivKey<K::MacKey, K::EncryptionKey>,
    mut ciphertext: Vec<u8>,
) -> Result<Vec<u8>, AesSivError> {
    // The ciphertext parameter has to consist of the integrity tag V, and then of at least 16 bytes
    // of actual ciphertext.
    if ciphertext.len() < 16 * 2 {
        return Err(AesSivError::CiphertextTooShort);
    }

    // Z is V || C, so the leading block is the V that encryption put there.
    let c = ciphertext.split_off(16);
    let v: [u8; 16] = ciphertext.try_into().unwrap();

    // Q = V bitand (1^64 || 0^1 || 1^31 || 0^1 || 1^31)
    // The AES-CTR counter block.
    let q = ctr_iv(v);

    // P = CTR(K2, Q, C)
    // Use `Zeroizing` for `plaintext`, so if the MAC check below fails, we don't leave the plaintext
    // around in memory. Note that this covers only this buffer and only the failure path: on
    // success the `to_vec()` at the end hands the caller an ordinary `Vec` that is theirs to clear.
    let plaintext = Zeroizing::new(
        backend
            .aes_ctr(&key.encryption_key, q, c)
            .await
            .map_err(|e| AesSivError::BackendError(e.into()))?,
    );

    // T = S2V(K1, AD1, ..., ADn, P), with n = 0, and the result is P only if T = V.
    //
    // This is the only thing standing between the caller and an attacker-chosen plaintext, so the
    // comparison has to be constant time: a byte-at-a-time one would let an attacker who can submit
    // ciphertexts and time the rejection forge a V one byte at a time. Hence subtle's ConstantTimeEq
    // below, whereas the natural `t != v` on two [u8; 16] is not constant time.
    let tag_matches: bool = s2v(backend, &key.mac_key, plaintext.as_slice()).await?.ct_eq(&v).into();
    if !tag_matches {
        return Err(AesSivError::AuthenticationFailed);
    }

    Ok(plaintext.to_vec())
}

/// S2V from [RFC 5297, section 2.4], over a single input string: derives the 128-bit `V` that is
/// both the integrity tag and the basis for the counter block, from the CMAC key `K1` and the
/// plaintext.
///
/// In full, S2V takes a list of input strings `S1, ..., Sn` and folds them together with a doubling
/// step per string. Only the case without associated data is implemented here, so that list holds
/// the plaintext and nothing else, `n` is 1, the folding loop over `S1, ..., Sn-1` collapses to
/// nothing, and what remains is the two AES-CMAC calls below.
///
/// [RFC 5297, section 2.4]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.4
async fn s2v<K: AesSivBackend>(backend: &K, key: &K::MacKey, s1: &[u8]) -> Result<[u8; 16], AesSivError> {
    // D = AES-CMAC(K, <zero>)
    // The all-zero block is on purpose: it is the fixed starting value the RFC specifies for D.
    // With n = 1 the loop over S1..Sn-1 does not run, so D is not doubled and S1 is also Sn.
    let d = backend
        .aes_cmac(key, vec![0; 16])
        .await
        .map_err(|e| AesSivError::BackendError(e.into()))?;

    // T = Sn xorend D
    // Everything but the last 16 bytes of `t` are the plaintext. We wrap that in `Zeroizing`,
    // so it is removed from memory after this function even in case of errors.
    let t = Zeroizing::new(xorend(s1, d)?);

    // return V = AES-CMAC(K, T)
    backend
        .aes_cmac(key, t)
        .await
        .map_err(|e| AesSivError::BackendError(e.into()))
}

/// The counter block that CTR mode starts from, per the SIV encryption construction in
/// [RFC 5297, section 2.6]: `Q = V bitand (1^64 || 0^1 || 1^31 || 0^1 || 1^31)`.
///
/// [RFC 5297, section 2.6]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.6
fn ctr_iv(v: [u8; 16]) -> [u8; 16] {
    (u128::from_be_bytes(v) & 0xffff_ffff_ffff_ffff_7fff_ffff_7fff_ffff).to_be_bytes()
}

/// `A xorend B` from [RFC 5297, section 2.1]: xoring `mask` onto the end of `value`, i.e.
/// `leftmost(A, len(A) - len(B)) || (rightmost(A, len(B)) xor B)`.
///
/// Only supports the case where len(A) >= 128 bits.
///
/// [RFC 5297, section 2.1]: https://datatracker.ietf.org/doc/html/rfc5297#section-2.1
fn xorend(value: &[u8], mask: [u8; 16]) -> Result<Vec<u8>, AesSivError> {
    if value.len() < 16 {
        return Err(AesSivError::PlaintextTooShort);
    }

    let (head, tail) = value.split_at(value.len() - 16);
    let tail = u128::from_be_bytes(tail.try_into().expect("tail is split off at exactly 16 bytes"));

    let mask = u128::from_be_bytes(mask);

    Ok([head, &(tail ^ mask).to_be_bytes()].concat())
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use hex_literal::hex;

    use crate::aes_siv::AesSivBackend;
    use crate::aes_siv::AesSivKey;
    use crate::aes_siv::aes_siv_decrypt;
    use crate::aes_siv::aes_siv_encrypt;

    /// A 512-bit AES-SIV key `K = K1 || K2`, to be taken apart with [`split_key()`].
    pub const KEY_A: [u8; 64] = hex!(
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
        "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f"
    );
    /// A second 512-bit key, unrelated to [`KEY_A`], for the cases that need two.
    pub const KEY_B: [u8; 64] = hex!(
        "fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0"
        "dfdedddcdbdad9d8d7d6d5d4d3d2d1d0cfcecdcccbcac9c8c7c6c5c4c3c2c1c0"
    );

    /// Splits one of the 512-bit keys above into the (K1, K2) pair that [`aes_siv_encrypt`] and
    /// [`aes_siv_decrypt`] take, in that argument order.
    ///
    /// [`aes_siv_encrypt`]: super::aes_siv_encrypt
    /// [`aes_siv_decrypt`]: super::aes_siv_decrypt
    pub fn split_key(key: [u8; 64]) -> ([u8; 32], [u8; 32]) {
        let cmac_key = key[..32].try_into().unwrap();
        let ctr_key = key[32..].try_into().unwrap();

        (cmac_key, ctr_key)
    }

    /// The AES-256 key that the NIST example documents use throughout, shared by
    /// [`AES_CMAC_TEST_CASES`] and the AES-CTR vector below.
    const NIST_KEY: [u8; 32] = hex!("603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4");

    /// The four CMAC-AES256 examples from NIST's [CMAC example values]. These used to be appendix D
    /// of [SP 800-38B], the document defining CMAC; the 2016 revision moved them out to that
    /// separate file, and appendix D is now a pointer to it.
    ///
    /// [CMAC example values]: https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/AES_CMAC.pdf
    /// [SP 800-38B]: https://doi.org/10.6028/NIST.SP.800-38B
    const AES_CMAC_TEST_CASES: [(&[u8], [u8; 16]); 4] = [
        (&hex!(""), hex!("028962f61b7bf89efc6b551f4667d983")),
        (
            &hex!("6bc1bee22e409f96e93d7e117393172a"),
            hex!("28a7023f452e8f82bd4bf28d8c37c35c"),
        ),
        (
            &hex!("6bc1bee22e409f96e93d7e117393172aae2d8a57"),
            hex!("156727dc0878944a023c1fe03bad6d93"),
        ),
        (
            &hex!(
                "6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e51"
                "30c81c46a35ce411e5fbc1191a0a52eff69f2445df4f9b17ad2b417be66c3710"
            ),
            hex!("e1992190549f6ed5696a2c056c315410"),
        ),
    ];

    struct AesCtrTestCase {
        /// The document's "Init. Counter".
        pub counter_block: [u8; 16],
        pub plaintext: &'static [u8],
        pub ciphertext: &'static [u8],
    }

    /// The CTR-AES256 example from [SP 800-38A], the document defining CTR mode: appendix F.5.5 is
    /// the encryption direction of the case below, F.5.6 the decryption direction.
    ///
    /// [SP 800-38A]: https://doi.org/10.6028/NIST.SP.800-38A
    const AES_CTR_TEST_CASE: AesCtrTestCase = AesCtrTestCase {
        counter_block: hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"),
        plaintext: &hex!(
            "6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e51"
            "30c81c46a35ce411e5fbc1191a0a52eff69f2445df4f9b17ad2b417be66c3710"
        ),
        ciphertext: &hex!(
            "601ec313775789a5b7a7f504bbf3d228f443e3ca4d62b59aca84e990cacaf5c5"
            "2b0930daa23de94ce87017ba2d84988ddfc9c58db67aada613c2dd08457941a6"
        ),
    };

    /// Holds a backend's [`aes_cmac()`](AesSivBackend::aes_cmac) to [`AES_CMAC_TEST_CASES`].
    ///
    /// AES-SIV is CTR and CMAC with S2V between them, and RFC 5297 has no vectors for the whole
    /// that apply here, so [`AES_SIV_TEST_CASES`] is cross-checked against another implementation
    /// rather than against a standard. This function and [`test_aes_ctr`] pin the two primitives to
    /// their own defining documents instead, leaving only the glue resting on a cross-check.
    ///
    /// `key_generator` turns the raw key bytes into whatever the backend uses to refer to keys.
    pub async fn test_aes_cmac<K, B>(backend: &B, key_generator: impl AsyncFn([u8; 32]) -> K)
    where
        B: AesSivBackend<MacKey = K>,
    {
        let key = key_generator(NIST_KEY).await;

        for (index, (message, expected)) in AES_CMAC_TEST_CASES.iter().enumerate() {
            let tag = backend.aes_cmac(&key, message.to_vec()).await.unwrap();

            assert_eq!(tag, *expected, "case {index}");
        }
    }

    /// Holds a backend's [`aes_ctr()`](AesSivBackend::aes_ctr) to [`AES_CTR_TEST_CASE`]; see
    /// [`test_aes_cmac`] for why.
    pub async fn test_aes_ctr<K, B>(backend: &B, key_generator: impl AsyncFn([u8; 32]) -> K)
    where
        B: AesSivBackend<EncryptionKey = K>,
    {
        let key = key_generator(NIST_KEY).await;

        let case = AES_CTR_TEST_CASE;

        // F.5.5, encrypting.
        let ciphertext = backend
            .aes_ctr(&key, case.counter_block, case.plaintext.to_vec())
            .await
            .unwrap();
        assert_eq!(ciphertext, case.ciphertext);

        // F.5.6, decrypting: the same case read the other way round. This is the same operation as
        // above, CTR being symmetric, but both directions are worth running because AES-SIV uses
        // this one function for both.
        let plaintext = backend
            .aes_ctr(&key, case.counter_block, case.ciphertext.to_vec())
            .await
            .unwrap();
        assert_eq!(plaintext, case.plaintext);
    }

    struct TestCase {
        pub key: [u8; 64],
        pub plaintext: &'static [u8],
        /// The full `Z = V || C`.
        pub expected: &'static [u8],
    }

    impl TestCase {
        /// The (K1, K2) pair of this case's key, as [`split_key()`] returns it.
        fn keys(&self) -> ([u8; 32], [u8; 32]) {
            split_key(self.key)
        }
    }

    /// Full `Z = V || C` for [`aes_siv_encrypt()`](super::aes_siv_encrypt) end to end, cross-checked
    /// against the RustCrypto "aes-siv" crate (RFC 5297 has no vectors for this).
    ///
    /// The leading 16 bytes of each `Z` below are the `V` values already pinned by `test_s2v`.
    const AES_SIV_TEST_CASES: [TestCase; 8] = [
        TestCase {
            key: KEY_A,
            plaintext: &hex!("112233445566778899aabbccddeeff00"),
            expected: &hex!("dfcd1b3b363f913fec392c3a9ef711dfa1f4fe6c9986e37ea6b5e75e03d478b2"),
        },
        TestCase {
            key: KEY_A,
            plaintext: &hex!("101112131415161718191a1b1c1d1e1f2021222324252627"),
            expected: &hex!("92f11ff1abf542c53342e2757de0098ec912ac7c551799cefe24283143ce945ce2b35f42c2c90e37"),
        },
        TestCase {
            key: KEY_A,
            plaintext: &hex!("00000000000000000000000000000000000000000000000000000000000000000000"),
            expected: &hex!(
                "1cd2b878630b7bbccfccf7602045f15a3937abc2000c6989a79fe5e36adb54c7fc46564b349df399fbb6acf63bc6eb1cbba7"
            ),
        },
        TestCase {
            key: KEY_A,
            plaintext: &hex!("74686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f67"),
            expected: &hex!(
                "215477c695fffb1d14e43bb018e4e204cdb75f0dbefe93a83694395ea8f9678a"
                "4adcb246ff83778d64570f6d7ef936b42a17e513f9326566639449"
            ),
        },
        TestCase {
            key: KEY_B,
            plaintext: &hex!("112233445566778899aabbccddeeff00"),
            expected: &hex!("b7e6dd3032146cc7e9868aec583f62e8c7407883d7523ddfa1e047ef40331b9c"),
        },
        TestCase {
            key: KEY_B,
            plaintext: &hex!("101112131415161718191a1b1c1d1e1f2021222324252627"),
            expected: &hex!("fc2261a9c2d144063301ce5898e9efc06752ab4877c38c47575b5c97e6b263236dd41247051049b9"),
        },
        TestCase {
            key: KEY_B,
            plaintext: &hex!("00000000000000000000000000000000000000000000000000000000000000000000"),
            expected: &hex!(
                "cf85b6c330dc2c219a34e95192a87be7a4b081f9ace4294ccd128e2cdc2ff0f9d02694252ce5fc031725db3ca10f85a62ea3"
            ),
        },
        TestCase {
            key: KEY_B,
            plaintext: &hex!("74686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f67"),
            expected: &hex!(
                "8c9ade993d3825bfcd5d39186746eed6acb6c13f31bd7ab1282d4bb1ce062a89"
                "64c86f01a8c24ef8e93c0bd4ff09d393b2101cd2642f4895ce4766"
            ),
        },
    ];

    /// Runs [`AES_SIV_TEST_CASES`] against `backend`, so that any [`AesSivBackend`] implementation
    /// can be held to the same known answers.
    ///
    /// `key_generator` turns the raw (K1, K2) bytes of a case into whatever the backend uses to refer to keys.
    pub async fn test_aes_siv_encrypt<K, B>(backend: &B, key_generator: impl AsyncFn(([u8; 32], [u8; 32])) -> (K, K))
    where
        K: Eq,
        B: AesSivBackend<MacKey = K, EncryptionKey = K>,
    {
        for (index, case) in AES_SIV_TEST_CASES.iter().enumerate() {
            let (cmac_key, ctr_key) = key_generator(case.keys()).await;

            let z = aes_siv_encrypt(
                backend,
                &AesSivKey::try_new(cmac_key, ctr_key).unwrap(),
                case.plaintext.to_vec(),
            )
            .await
            .unwrap();

            assert_eq!(z, case.expected, "case {index}");
            assert_eq!(z.len(), 16 + case.plaintext.len(), "case {index}");
        }
    }

    /// The same pinned vectors as [`test_aes_siv_encrypt`], read in the other direction. Testing decrypt
    /// against a round trip alone would pass even if both directions shared a compensating error, so
    /// the ciphertexts here are the externally generated ones rather than whatever encrypt produced.
    pub async fn test_aes_siv_decrypt<K, B>(backend: &B, key_generator: impl AsyncFn(([u8; 32], [u8; 32])) -> (K, K))
    where
        K: Eq,
        B: AesSivBackend<MacKey = K, EncryptionKey = K>,
    {
        for (index, case) in AES_SIV_TEST_CASES.iter().enumerate() {
            let (cmac_key, ctr_key) = key_generator(case.keys()).await;

            let plaintext = aes_siv_decrypt(
                backend,
                &AesSivKey::try_new(cmac_key, ctr_key).unwrap(),
                case.expected.to_vec(),
            )
            .await
            .unwrap();

            assert_eq!(plaintext, case.plaintext, "case {index}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use aes::Aes256;
    use aes::cipher::KeyIvInit;
    use aes::cipher::StreamCipher;
    use cmac::Cmac;
    use cmac::Mac;
    use ctr::Ctr128BE;
    use hex_literal::hex;
    use rstest::rstest;

    use super::AesSivBackend;
    use super::AesSivError;
    use super::AesSivKey;
    use super::AesSivKeysEqualError;
    use super::aes_siv_decrypt;
    use super::aes_siv_encrypt;
    use super::ctr_iv;
    use super::s2v;
    use super::test::KEY_A;
    use super::test::KEY_B;
    use super::test::split_key;
    use super::xorend;

    /// An [`AesSivBackend`] that performs both operations in memory, on raw keys handed to it by
    /// the caller, for tests and for local development.
    pub struct MemoryAesSivBackend;

    impl AesSivBackend for MemoryAesSivBackend {
        type Error = Infallible;

        type MacKey = [u8; 32];
        type EncryptionKey = [u8; 32];

        async fn aes_cmac(
            &self,
            key: &Self::MacKey,
            value: impl AsRef<[u8]> + Send + 'static,
        ) -> Result<[u8; 16], Self::Error> {
            let mut mac = Cmac::<Aes256>::new(key.into());
            mac.update(value.as_ref());
            let result = mac.finalize().into_bytes().into();
            Ok(result)
        }

        async fn aes_ctr(
            &self,
            key: &Self::EncryptionKey,
            counter_block: [u8; 16],
            input: impl AsRef<[u8]> + Send + 'static,
        ) -> Result<Vec<u8>, Self::Error> {
            let mut cipher = Ctr128BE::<Aes256>::new(key.into(), &counter_block.into());
            let input = input.as_ref();
            let mut output = vec![0; input.len()];
            cipher
                .apply_keystream_b2b(input, output.as_mut_slice())
                .expect("buffers are the same length");

            Ok(output)
        }
    }

    #[rstest]
    // The one worked xorend in RFC 5297, from the S2V of appendix A.2: its 47-byte plaintext
    // ("this is some plaintext to encrypt using SIV-AES") xorended with the D that S2V holds after
    // folding in the two associated data strings and the nonce, which is the "xor" line directly
    // above the "xorend" line there. Everything but the last block is passed through untouched,
    // which here is the leading 31 bytes.
    #[case(
        &hex!("7468697320697320736f6d6520706c61696e7465787420746f20656e6372797074207573696e67205349562d414553"),
        hex!("16592c17729a5a725567636168b48376"),
        &hex!("7468697320697320736f6d6520706c61696e7465787420746f20656e637279662d0c6201f3341575342a3745f5c625")
    )]
    // Exactly one block, so there is nothing to pass through and xorend degenerates to xor. An
    // all-ones mask makes that complement the input, which is checkable by eye.
    #[case(
        &hex!("112233445566778899aabbccddeeff00"),
        hex!("ffffffffffffffffffffffffffffffff"),
        &hex!("eeddccbbaa99887766554433221100ff")
    )]
    // A zero mask leaves the input alone.
    #[case(
        &hex!("112233445566778899aabbccddeeff0011223344"),
        hex!("00000000000000000000000000000000"),
        &hex!("112233445566778899aabbccddeeff0011223344")
    )]
    fn test_xorend(#[case] value: &[u8], #[case] mask: [u8; 16], #[case] expected: &[u8]) {
        assert_eq!(xorend(value, mask).unwrap(), expected);
    }

    // For S2V, RFC 5297 contains no test vectors that apply. Both worked examples in appendix A use a
    // 256-bit key, i.e. AES-SIV-CMAC-256 over AES-128 CMAC rather than the AES-256 CMAC used here,
    // and both call S2V with more than one string (2 in A.1, 4 in A.2), so neither reaches this
    // n = 1 path. These vectors were instead created using the RustCrypto "aes-siv" crate,
    // whose Aes256Siv is CmacSiv<Aes256> and whose detached tag is exactly the S2V output.
    // Encrypting with no AD makes S2V take a single string, so the tag for key K1 || K2 and
    // plaintext P equals s2v(K1, P).
    #[rstest]
    #[case(
        hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"),
        &hex!("112233445566778899aabbccddeeff00"),
        hex!("dfcd1b3b363f913fec392c3a9ef711df")
    )]
    #[case(
        hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"),
        &hex!("101112131415161718191a1b1c1d1e1f2021222324252627"),
        hex!("92f11ff1abf542c53342e2757de0098e")
    )]
    #[case(
        hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"),
        &hex!("74686520717569636b2062726f776e20666f78206a756d7073206f76657220746865206c617a7920646f67"),
        hex!("215477c695fffb1d14e43bb018e4e204")
    )]
    #[case(
        hex!("fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0"),
        &hex!("112233445566778899aabbccddeeff00"),
        hex!("b7e6dd3032146cc7e9868aec583f62e8")
    )]
    #[case(
        hex!("fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0efeeedecebeae9e8e7e6e5e4e3e2e1e0"),
        &hex!("00000000000000000000000000000000000000000000000000000000000000000000"),
        hex!("cf85b6c330dc2c219a34e95192a87be7")
    )]
    #[tokio::test]
    async fn test_s2v(#[case] key: [u8; 32], #[case] s1: &[u8], #[case] expected: [u8; 16]) {
        assert_eq!(s2v(&MemoryAesSivBackend, &key, s1).await.unwrap(), expected);
    }

    #[rstest]
    // RFC 5297, appendix A.1.
    #[case(hex!("85632d07c6e8f37f950acd320a2ecc93"), hex!("85632d07c6e8f37f150acd320a2ecc93"))]
    // RFC 5297, appendix A.2.
    #[case(hex!("7bdb6e3b432667eb06f4d14bff2fbd0f"), hex!("7bdb6e3b432667eb06f4d14b7f2fbd0f"))]
    // Every bit set, so only the two masked bits may change, and nothing set at all.
    #[case(hex!("ffffffffffffffffffffffffffffffff"), hex!("ffffffffffffffff7fffffff7fffffff"))]
    #[case(hex!("00000000000000000000000000000000"), hex!("00000000000000000000000000000000"))]
    fn test_ctr_iv(#[case] v: [u8; 16], #[case] expected: [u8; 16]) {
        assert_eq!(ctr_iv(v), expected);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(16 - 1)]
    #[tokio::test]
    async fn test_aes_siv_rejects_short_plaintext(#[case] len: usize) {
        let error = aes_siv_encrypt(
            &MemoryAesSivBackend,
            &AesSivKey::try_new([0; 32], [1; 32]).unwrap(),
            vec![0; len],
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AesSivError::PlaintextTooShort));
    }

    /// Async version of [`std::convert::identity`].
    async fn identity<T>(val: T) -> T {
        val
    }

    // In these tests the in-memory backend takes and uses the raw key bytes directly.
    #[tokio::test]
    async fn test_aes_cmac() {
        super::test::test_aes_cmac(&MemoryAesSivBackend, identity).await
    }
    #[tokio::test]
    async fn test_aes_ctr() {
        super::test::test_aes_ctr(&MemoryAesSivBackend, identity).await
    }
    #[tokio::test]
    async fn test_aes_siv_encrypt() {
        super::test::test_aes_siv_encrypt(&MemoryAesSivBackend, identity).await
    }
    #[tokio::test]
    async fn test_aes_siv_decrypt() {
        super::test::test_aes_siv_decrypt(&MemoryAesSivBackend, identity).await
    }

    #[rstest]
    #[case(16)]
    #[case(16 + 3)]
    #[case(16 * 3)]
    #[tokio::test]
    async fn test_aes_siv_round_trip(#[case] len: usize) {
        let (cmac_key, ctr_key) = split_key(KEY_A);
        let key = AesSivKey::try_new(cmac_key, ctr_key).unwrap();
        let plaintext: Vec<u8> = (0..len).map(|i| i as u8).collect();

        let ciphertext = aes_siv_encrypt(&MemoryAesSivBackend, &key, plaintext.clone())
            .await
            .unwrap();

        assert_eq!(
            aes_siv_decrypt(&MemoryAesSivBackend, &key, ciphertext).await.unwrap(),
            plaintext
        );
    }

    // Flipping any single bit anywhere in Z has to be rejected, whether it lands in V or in C.
    #[rstest]
    #[case(0)]
    #[case(16 - 1)]
    #[case(16)]
    #[case(16 * 3 - 1)]
    #[tokio::test]
    async fn test_aes_siv_decrypt_rejects_tampering(#[case] index: usize) {
        let (cmac_key, ctr_key) = split_key(KEY_A);
        let key = AesSivKey::try_new(cmac_key, ctr_key).unwrap();

        let mut ciphertext = aes_siv_encrypt(&MemoryAesSivBackend, &key, vec![0; 16 * 2])
            .await
            .unwrap();
        ciphertext[index] ^= 1;

        let error = aes_siv_decrypt(&MemoryAesSivBackend, &key, ciphertext)
            .await
            .unwrap_err();

        assert!(matches!(error, AesSivError::AuthenticationFailed));
    }

    // Since the key consists of two halves, "the wrong key" covers three cases: either one of them
    // wrong on its own, or both. All three have to be rejected, including the one where the CMAC key
    // still matches and only the recovered plaintext is garbage.
    #[rstest]
    #[case(true, false)]
    #[case(false, true)]
    #[case(true, true)]
    #[tokio::test]
    async fn test_aes_siv_decrypt_rejects_wrong_key(#[case] wrong_cmac_key: bool, #[case] wrong_ctr_key: bool) {
        let (cmac_key, ctr_key) = split_key(KEY_A);
        let (other_cmac_key, other_ctr_key) = split_key(KEY_B);

        let ciphertext = aes_siv_encrypt(
            &MemoryAesSivBackend,
            &AesSivKey::try_new(cmac_key, ctr_key).unwrap(),
            vec![0; 16 * 2],
        )
        .await
        .unwrap();

        let wrong_key = AesSivKey::try_new(
            if wrong_cmac_key { other_cmac_key } else { cmac_key },
            if wrong_ctr_key { other_ctr_key } else { ctr_key },
        )
        .unwrap();

        let error = aes_siv_decrypt(&MemoryAesSivBackend, &wrong_key, ciphertext)
            .await
            .unwrap_err();

        assert!(matches!(error, AesSivError::AuthenticationFailed));
    }

    #[rstest]
    #[case(0)]
    #[case(16)]
    #[case(16 * 2 - 1)]
    #[tokio::test]
    async fn test_aes_siv_decrypt_rejects_short_ciphertext(#[case] len: usize) {
        let error = aes_siv_decrypt(
            &MemoryAesSivBackend,
            &AesSivKey::try_new([0; 32], [1; 32]).unwrap(),
            vec![0; len],
        )
        .await
        .unwrap_err();

        assert!(matches!(error, AesSivError::CiphertextTooShort));
    }

    #[test]
    fn test_aes_siv_rejects_identical_keys() {
        assert!(matches!(
            AesSivKey::try_new([0; 32], [0; 32]).unwrap_err(),
            AesSivKeysEqualError
        ));
    }
}
