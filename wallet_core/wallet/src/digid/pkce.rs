const CODE_VERIFIER_LENGTH: usize = 128;

pub fn generate_verifier_and_challenge() -> (String, String) {
    // Generate a random 128-byte code verifier (must be between 43 and 128 bytes)
    let code_verifier = pkce::code_verifier(CODE_VERIFIER_LENGTH);

    // Generate an encrypted code challenge accordingly
    let code_challenge = pkce::code_challenge(&code_verifier);

    // Generate PKCE verifier
    let pkce_verifier = String::from_utf8(code_verifier).expect("Generated PKCE verifier is not valid UTF-8");

    (pkce_verifier, code_challenge)
}
