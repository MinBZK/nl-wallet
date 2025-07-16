use std::env;
use std::path::PathBuf;
use std::time::Duration;

use config::Config;
use config::ConfigError;
use config::Environment;
use config::File;
use serde::Deserialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub database: Database,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct ConnectionOptions {
    #[serde(rename = "connect_timeout_in_sec")]
    #[serde_as(as = "DurationSeconds")]
    pub connect_timeout: Duration,

    pub max_connections: u8,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            max_connections: 10,
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Database {
    pub host: String,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub connection_options: ConnectionOptions,
}

impl Default for Database {
    fn default() -> Self {
        Self {
            host: String::from("localhost"),
            name: String::from("wallet_provider"),
            username: Some(String::from("postgres")),
            password: Some(String::from("postgres")),
            connection_options: Default::default(),
        }
    }
}

pub type ConnectionString = String;

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is one directory up from the Cargo.toml for this crate if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .ok()
            .and_then(|path| path.parent().map(|path| path.to_path_buf()))
            .unwrap_or_default();

        Config::builder()
            .add_source(File::from(config_path.join("wallet_provider.toml")).required(false))
            .add_source(
                Environment::with_prefix("wallet_provider")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()?
            .try_deserialize()
    }
}

impl Database {
    pub fn connection_string(&self) -> ConnectionString {
        let username_password = self
            .username
            .as_ref()
            .map(|u| {
                format!(
                    "{}{}@",
                    u,
                    self.password.as_ref().map(|p| format!(":{p}")).unwrap_or_default()
                )
            })
            .unwrap_or_default();

        format!("postgres://{}{}/{}", username_password, self.host, self.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;

    fn db(host: &str, db_name: &str, username: Option<&str>, password: Option<&str>) -> Database {
        Database {
            host: host.to_string(),
            name: db_name.to_string(),
            username: username.map(String::from),
            password: password.map(String::from),
            connection_options: Default::default(),
        }
    }

    #[test]
    fn test_connection_string() {
        assert_eq!(
            db("host", "db", Some("user"), Some("pwd")).connection_string(),
            "postgres://user:pwd@host/db"
        );
        assert_eq!(db("host", "db", None, None).connection_string(), "postgres://host/db");
        assert_eq!(
            db("host", "db", Some("user"), None).connection_string(),
            "postgres://user@host/db"
        );
        assert_eq!(
            db("host", "db", None, Some("pwd")).connection_string(),
            "postgres://host/db"
        );
    }
}
