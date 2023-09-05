mod client;

use async_trait::async_trait;
use reqwest::StatusCode;
use url::{ParseError, Url};

use wallet_common::account::{
    messages::{
        auth::{Registration, WalletCertificate},
        errors::ErrorData,
        instructions::{CheckPin, Instruction, InstructionChallengeRequestMessage, InstructionResult},
    },
    signed::SignedDouble,
};

pub use self::client::AccountServerClient;

#[derive(Debug, thiserror::Error)]
pub enum AccountProviderError {
    #[error("server responded with {0}")]
    Response(#[from] AccountProviderResponseError),
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountProviderResponseError {
    #[error("status code {0}")]
    Status(StatusCode),
    #[error("status code {0} and contents: {1}")]
    Text(StatusCode, String),
    #[error("status code {0} and error: {1}")]
    Data(StatusCode, ErrorData),
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait AccountProvider {
    async fn registration_challenge(&self, base_url: &Url) -> Result<Vec<u8>, AccountProviderError>;

    async fn register(
        &self,
        base_url: &Url,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountProviderError>;

    async fn instruction_challenge(
        &self,
        base_url: &Url,
        challenge_request: InstructionChallengeRequestMessage,
    ) -> Result<Vec<u8>, AccountProviderError>;

    async fn check_pin(
        &self,
        base_url: &Url,
        instruction: Instruction<CheckPin>,
    ) -> Result<InstructionResult<()>, AccountProviderError>;
}
