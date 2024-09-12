mod client;

use reqwest::StatusCode;
use url::ParseError;

use error_category::ErrorCategory;
use wallet_common::{
    account::{
        messages::{
            auth::{Registration, WalletCertificate},
            errors::{AccountError, AccountErrorType},
            instructions::{Instruction, InstructionChallengeRequest, InstructionEndpoint, InstructionResult},
        },
        signed::ChallengeResponse,
    },
    urls::BaseUrl,
};

pub use self::client::HttpAccountProviderClient;

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum AccountProviderError {
    #[error("server responded with {0}")]
    Response(#[from] AccountProviderResponseError),
    #[error("networking error: {0}")]
    #[category(expected)]
    Networking(#[from] reqwest::Error),
    #[error("could not parse base URL: {0}")]
    #[category(pd)]
    BaseUrl(#[from] ParseError),
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum AccountProviderResponseError {
    #[error("status code {0}")]
    #[category(critical)]
    Status(StatusCode),
    #[error("status code {0} and contents: {1}")]
    Text(StatusCode, String),
    #[error("error with type and detail: ({}) {}", AccountErrorType::from(.0), .1.as_deref().unwrap_or("<NO DETAIL>"))]
    #[category(defer)]
    Account(#[defer] AccountError, Option<String>),
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AccountProviderClient {
    async fn registration_challenge(&self, base_url: &BaseUrl) -> Result<Vec<u8>, AccountProviderError>;

    async fn register(
        &self,
        base_url: &BaseUrl,
        registration_message: ChallengeResponse<Registration>,
    ) -> Result<WalletCertificate, AccountProviderError>;

    async fn instruction_challenge(
        &self,
        base_url: &BaseUrl,
        challenge_request: InstructionChallengeRequest,
    ) -> Result<Vec<u8>, AccountProviderError>;

    async fn instruction<I>(
        &self,
        base_url: &BaseUrl,
        instruction: Instruction<I>,
    ) -> Result<InstructionResult<I::Result>, AccountProviderError>
    where
        I: InstructionEndpoint + 'static;
}
