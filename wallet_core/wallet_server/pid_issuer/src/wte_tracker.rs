use jwt::credential::JwtCredentialClaims;
use jwt::wte::WteClaims;
use jwt::VerifiedJwt;
use openid4vc::server_state::MemoryWteTracker;
use openid4vc::server_state::WteTracker;
use server_utils::store::DatabaseConnection;
use server_utils::store::DatabaseError;

#[cfg(feature = "postgres")]
pub use postgres::PostgresWteTracker;

pub enum WteTrackerVariant {
    #[cfg(feature = "postgres")]
    Postgres(PostgresWteTracker),
    Memory(MemoryWteTracker),
}

impl WteTrackerVariant {
    pub fn new(connection: DatabaseConnection) -> Self {
        match connection {
            #[cfg(feature = "postgres")]
            DatabaseConnection::Postgres(connection) => Self::Postgres(PostgresWteTracker::new(connection)),
            DatabaseConnection::Memory => Self::Memory(MemoryWteTracker::new()),
        }
    }
}

impl WteTracker for WteTrackerVariant {
    type Error = DatabaseError;

    async fn track_wte(&self, wte: &VerifiedJwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WteTrackerVariant::Postgres(postgres_wte_tracker) => Ok(postgres_wte_tracker.track_wte(wte).await?),
            WteTrackerVariant::Memory(memory_wte_tracker) => {
                Ok(memory_wte_tracker.track_wte(wte).await.unwrap()) // this implementation is infallible
            }
        }
    }

    async fn cleanup(&self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "postgres")]
            WteTrackerVariant::Postgres(postgres_wte_tracker) => Ok(postgres_wte_tracker.cleanup().await?),
            WteTrackerVariant::Memory(memory_wte_tracker) => {
                memory_wte_tracker.cleanup().await.unwrap(); // this implementation is infallible
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
    use jwt::credential::JwtCredentialClaims;
    use jwt::wte::WteClaims;
    use jwt::VerifiedJwt;
    use server_utils::entity::used_wtes;
    use wallet_common::generator::Generator;
    use wallet_common::generator::TimeGenerator;

    use openid4vc::server_state::WteTracker;

    pub struct PostgresWteTracker<G = TimeGenerator> {
        time: G,
        connection: DatabaseConnection,
    }

    impl<G> PostgresWteTracker<G> {
        pub fn new_with_time(connection: DatabaseConnection, time: G) -> Self {
            Self { time, connection }
        }
    }

    impl PostgresWteTracker {
        pub fn new(connection: DatabaseConnection) -> Self {
            Self::new_with_time(connection, TimeGenerator)
        }
    }

    impl<G> WteTracker for PostgresWteTracker<G>
    where
        G: Generator<DateTime<Utc>> + Send + Sync,
    {
        type Error = DbErr;

        async fn track_wte(&self, wte: &VerifiedJwt<JwtCredentialClaims<WteClaims>>) -> Result<bool, Self::Error> {
            let shasum = sha256(wte.jwt().0.as_bytes());
            let expires = wte.payload().contents.attributes.exp;

            let query_result = used_wtes::Entity::insert(used_wtes::ActiveModel {
                used_wte_hash: ActiveValue::set(shasum),
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

            match used_wtes::Entity::delete_many()
                .filter(used_wtes::Column::Expires.lte(now))
                .exec(&self.connection)
                .await
            {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
    }
}
