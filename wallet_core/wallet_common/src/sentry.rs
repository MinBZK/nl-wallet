use std::borrow::Cow;

use sentry::{release_name, ClientInitGuard, ClientOptions};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Sentry {
    pub dsn: String,
    pub environment: Option<String>,
}

impl Sentry {
    /// Initialize Sentry and return the [`ClientInitGuard`].
    #[must_use = "this value must be retained during the lifetime of the application"]
    pub fn init(&self) -> ClientInitGuard {
        sentry::init((
            self.dsn.clone(),
            ClientOptions {
                release: release_name!(),
                environment: self.environment.as_ref().map(|e| Cow::from(e.clone())),
                debug: cfg!(debug_assertions),
                ..Default::default()
            },
        ))
    }
}
