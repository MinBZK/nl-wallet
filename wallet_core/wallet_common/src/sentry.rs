use std::{borrow::Cow, error::Error};

use sentry::{protocol::Event, ClientInitGuard, ClientOptions, Level};
use serde::Deserialize;
use uuid::Uuid;

use crate::error_category::{Category, ErrorCategory};

#[derive(Clone, Deserialize)]
pub struct Sentry {
    pub dsn: String,
    pub environment: String,
}

impl Sentry {
    /// Initialize Sentry and return the [`ClientInitGuard`].
    /// [release] must be set to `sentry::release_name!()` from the `main.rs` of the application, so that the release
    /// matches the main crate.
    #[must_use = "this value must be retained during the lifetime of the application"]
    pub fn init(&self, release: Option<Cow<'static, str>>) -> ClientInitGuard {
        sentry::init((
            self.dsn.clone(),
            ClientOptions {
                release,
                environment: Cow::from(self.environment.clone()).into(),
                debug: cfg!(debug_assertions),
                ..Default::default()
            },
        ))
    }
}

pub fn classify_and_report_to_sentry<T: ErrorCategory + Error>(error: T) -> T {
    match error.category() {
        Category::Expected => {}
        Category::Critical => {
            let _uuid = sentry::capture_error(&error);
        }
        Category::PersonalData => {
            let event = Event {
                event_id: Uuid::new_v4(),
                message: Some(std::any::type_name_of_val(&error).to_string()),
                level: Level::Error,
                ..Default::default()
            };
            let _uuid = sentry::capture_event(event);
        }
    }
    error
}
