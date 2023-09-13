mod remote;

#[cfg(test)]
mod mock;

use async_trait::async_trait;
use reqwest::StatusCode;
use url::{ParseError, Url};

use wallet_common::account::{
    messages::{
        auth::{Registration, WalletCertificate},
        errors::ErrorData,
        instructions::{Instruction, InstructionChallengeRequestMessage, InstructionEndpoint, InstructionResult},
    },
    signed::SignedDouble,
};

pub use self::remote::RemoteAccountServerClient;

#[derive(Debug, thiserror::Error)]
pub enum AccountServerClientError {
    #[error("server responded with {0}")]
    Response(#[from] AccountServerResponseError),
    #[error("networking error: {0}")]
    Networking(#[from] reqwest::Error),
    #[error("could not parse base URL: {0}")]
    BaseUrl(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum AccountServerResponseError {
    #[error("status code {0}")]
    Status(StatusCode),
    #[error("status code {0} and contents: {1}")]
    Text(StatusCode, String),
    #[error("status code {0} and error: {1}")]
    Data(StatusCode, ErrorData),
}

#[async_trait]
pub trait AccountServerClient {
    async fn new(base_url: &Url) -> Self
    where
        Self: Sized;

    async fn registration_challenge(&self) -> Result<Vec<u8>, AccountServerClientError>;

    async fn register(
        &self,
        registration_message: SignedDouble<Registration>,
    ) -> Result<WalletCertificate, AccountServerClientError>;

    async fn instruction_challenge(
        &self,
        challenge_request: InstructionChallengeRequestMessage,
    ) -> Result<Vec<u8>, AccountServerClientError>;

    async fn instruction<I>(
        &self,
        instruction: Instruction<I>,
    ) -> Result<InstructionResult<I::Result>, AccountServerClientError>
    where
        I: InstructionEndpoint + Send + Sync;
}
