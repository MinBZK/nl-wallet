use derive_more::Constructor;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::SelectColumns;
use token_status_list::status_list_service::RevocationError;
use uuid::Uuid;

use super::PostgresStatusListService;
use crate::entity::attestation_batch;

#[derive(Debug)]
#[cfg_attr(feature = "test_api", derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema))]
pub struct BatchIsRevoked {
    pub batch_id: Uuid,
    pub is_revoked: bool,
}

#[derive(Debug, Clone, Constructor)]
pub struct PostgresRevocationHelper {
    connection: DatabaseConnection,
}

impl PostgresRevocationHelper {
    pub fn from_status_list<K, R>(status_list_service: &PostgresStatusListService<K, R>) -> Self {
        Self {
            connection: status_list_service.connection.clone(),
        }
    }

    pub async fn get_attestation_batch(&self, batch_id: Uuid) -> Result<BatchIsRevoked, RevocationError> {
        attestation_batch::Entity::find()
            .filter(attestation_batch::Column::BatchId.eq(batch_id))
            .select_only()
            .select_column(attestation_batch::Column::BatchId)
            .select_column(attestation_batch::Column::IsRevoked)
            .into_tuple()
            .one(&self.connection)
            .await
            .map_err(|error| RevocationError::InternalError(Box::new(error)))?
            .map(|(batch_id, is_revoked)| BatchIsRevoked { batch_id, is_revoked })
            .ok_or_else(|| RevocationError::BatchIdNotFound(batch_id))
    }

    pub async fn list_attestation_batches(&self) -> Result<Vec<BatchIsRevoked>, RevocationError> {
        Ok(attestation_batch::Entity::find()
            .select_only()
            .select_column(attestation_batch::Column::BatchId)
            .select_column(attestation_batch::Column::IsRevoked)
            .into_tuple()
            .all(&self.connection)
            .await
            .map_err(|error| RevocationError::InternalError(Box::new(error)))?
            .into_iter()
            .map(|(batch_id, is_revoked)| BatchIsRevoked { batch_id, is_revoked })
            .collect())
    }
}
