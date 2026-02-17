#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("connection error: {0}")]
    Connection(#[source] sea_orm::DbErr),

    #[error("transaction error: {0}")]
    Transaction(#[source] sea_orm::DbErr),

    #[error("execution error: {0}")]
    Execution(#[source] sea_orm::DbErr),

    #[error("verifying key conversion error: {0}")]
    VerifyingKeyConversion(#[from] p256::pkcs8::spki::Error),

    #[error("signing key conversion error: {0}")]
    SigningKeyConversion(#[from] p256::ecdsa::Error),

    #[error("error converting to SemVer: {0}")]
    SemVerConversion(#[from] semver::Error),

    #[error("no rows were updated")]
    NoRowsUpdated,
}
