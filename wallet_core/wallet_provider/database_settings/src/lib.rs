use std::{env, path::PathBuf};

use config::{builder::BuilderState, Config, ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: Database,
}

#[derive(Deserialize)]
pub struct Database {
    pub host: String,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub type ConnectionString = String;

pub trait DatabaseDefaults<T: BuilderState> {
    fn database_defaults(self) -> Result<ConfigBuilder<T>, ConfigError>;
}

impl<T> DatabaseDefaults<T> for ConfigBuilder<T>
where
    T: BuilderState,
{
    fn database_defaults(self) -> Result<ConfigBuilder<T>, ConfigError> {
        self.set_default("database.host", "localhost")?
            .set_default("database.name", "wallet_provider")?
            .set_default("database.username", "postgres")?
            .set_default("database.password", "postgres")
    }
}

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
            .database_defaults()?
            .add_source(File::from(config_path.join("wallet_provider.toml")).required(false))
            .add_source(
                Environment::with_prefix("wallet_provider")
                    .separator("__")
                    .prefix_separator("_"),
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
                    self.password.as_ref().map(|p| format!(":{}", p)).unwrap_or_default()
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
