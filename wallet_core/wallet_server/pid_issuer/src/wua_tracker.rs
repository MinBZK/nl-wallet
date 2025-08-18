use jwt::VerifiedJwt;
use jwt::credential::JwtCredentialClaims;
use jwt::wua::WuaClaims;
use openid4vc::server_state::MemoryWuaTracker;
use openid4vc::server_state::WuaTracker;
use server_utils::store::DatabaseConnection;
use server_utils::store::DatabaseError;

#[cfg(feature = "postgres")]
pub use postgres::PostgresWuaTracker;

pub enum WuaTrackerVariant {
    #[cfg(feature = "postgres")]
    Postgres(PostgresWuaTracker),
    Memory(MemoryWuaTracker),
}

impl WuaTrackerVariant {
    pub fn new(connection: DatabaseConnection) -> Self {
        match connection {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(connection) => Self::Postgres(PostgresWuaTracker::new(connection)),
            DatabaseConnection::Memory => Self::Memory(MemoryWuaTracker::new()),
        }
    }
}

impl WuaTracker for WuaTrackerVariant {
    type Error = DatabaseError;

    async fn track_wua(&self, wua: &VerifiedJwt<JwtCredentialClaims<WuaClaims>>) -> Result<bool, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WuaTrackerVariant::Postgres(postgres_wua_tracker) => Ok(postgres_wua_tracker.track_wua(wua).await?),
            WuaTrackerVariant::Memory(memory_wua_tracker) => {
                Ok(memory_wua_tracker.track_wua(wua).await.unwrap()) // this implementation is infallible
            }
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WuaTrackerVariant::Postgres(postgres_wua_tracker) => Ok(postgres_wua_tracker.cleanup().await?),
            WuaTrackerVariant::Memory(memory_wua_tracker) => {
                memory_wua_tracker.cleanup().await.unwrap(); // this implementation is infallible
                Ok(())
            }
        }
    }
}

#[cfg(feature = "postgres")]
mod postgres {
    use chrono::DateTime;
    use chrono::Utc;
    use sea_orm::ActiveValue;
    use sea_orm::ColumnTrait;
    use sea_orm::DatabaseConnection;
    use sea_orm::DbErr;
    use sea_orm::EntityTrait;
    use sea_orm::QueryFilter;
    use sea_orm::SqlErr;

    use crypto::utils::sha256;
    use jwt::VerifiedJwt;
    use jwt::credential::JwtCredentialClaims;
    use jwt::wua::WuaClaims;
    use server_utils::entity::used_wuas;
    use utils::generator::Generator;
    use utils::generator::TimeGenerator;

    use openid4vc::server_state::WuaTracker;

    pub struct PostgresWuaTracker<G = TimeGenerator> {
        time: G,
        connection: DatabaseConnection,
    }

    impl<G> PostgresWuaTracker<G> {
        pub fn new_with_time(connection: DatabaseConnection, time: G) -> Self {
            Self { time, connection }
        }
    }

    impl PostgresWuaTracker {
        pub fn new(connection: DatabaseConnection) -> Self {
            Self::new_with_time(connection, TimeGenerator)
        }
    }

    impl<G> WuaTracker for PostgresWuaTracker<G>
    where
        G: Generator<DateTime<Utc>> + Send + Sync,
    {
        type Error = DbErr;

        async fn track_wua(&self, wua: &VerifiedJwt<JwtCredentialClaims<WuaClaims>>) -> Result<bool, Self::Error> {
            let shasum = sha256(wua.jwt().0.as_bytes());
            let expires = wua.payload().contents.attributes.exp;

            let query_result = used_wuas::Entity::insert(used_wuas::ActiveModel {
                used_wua_hash: ActiveValue::set(shasum),
                expires: ActiveValue::set(expires.into()),
            })
            .exec(&self.connection)
            .await;

            match query_result {
                Ok(_) => Ok(false),
                Err(err) if matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) => Ok(true),
                Err(err) => Err(err),
            }
        }

        async fn cleanup(&self) -> Result<(), Self::Error> {
            let now = self.time.generate();

            match used_wuas::Entity::delete_many()
                .filter(used_wuas::Column::Expires.lte(now))
                .exec(&self.connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
    }
}
