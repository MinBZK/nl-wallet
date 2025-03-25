use crate::sd_jwt::SdJwt;

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-simple-structured-sd-jwt
pub const SIMPLE_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/simple_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-complex-structured-sd-jwt
pub const COMPLEX_STRUCTURED_SD_JWT: &str = include_str!("../examples/sd_jwt/complex_structured.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-sd-jwt-based-verifiable-cre
pub const SD_JWT_VC: &str = include_str!("../examples/sd_jwt/sd_jwt_vc.jwt");

// Taken from https://www.ietf.org/archive/id/draft-ietf-oauth-selective-disclosure-jwt-17.html#name-presentation
pub const WITH_KB_SD_JWT: &str = include_str!("../examples/sd_jwt/with_kb.jwt");

pub fn simple_structured_sd_jwt() -> SdJwt {
    SdJwt::parse(SIMPLE_STRUCTURED_SD_JWT).unwrap()
}

pub fn complex_structured_sd_jwt() -> SdJwt {
    SdJwt::parse(COMPLEX_STRUCTURED_SD_JWT).unwrap()
}

pub fn sd_jwt_vc() -> SdJwt {
    SdJwt::parse(SD_JWT_VC).unwrap()
}

pub fn sd_jwt_kb() -> SdJwt {
    SdJwt::parse(WITH_KB_SD_JWT).unwrap()
}
