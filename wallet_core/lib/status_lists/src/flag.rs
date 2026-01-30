use derive_more::Constructor;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::SelectColumns;
use sea_orm::Set;
use sea_orm::sea_query::OnConflict;

use crate::entity::status_list_flag;

#[derive(Debug, Clone, Constructor)]
pub struct Flag {
    connection: DatabaseConnection,
    name: String,
}

impl Flag {
    pub async fn is_set(&self) -> Result<bool, DbErr> {
        let result: Option<bool> = status_list_flag::Entity::find()
            .select_only()
            .select_column(status_list_flag::Column::Value)
            .filter(status_list_flag::Column::Name.eq(&self.name))
            .into_tuple()
            .one(&self.connection)
            .await?;

        Ok(result.unwrap_or(false))
    }

    pub async fn set(&self) -> Result<(), DbErr> {
        let model = status_list_flag::ActiveModel {
            name: Set(self.name.clone()),
            value: Set(true),
        };
        status_list_flag::Entity::insert(model)
            .on_conflict(
                OnConflict::column(status_list_flag::Column::Name)
                    .update_column(status_list_flag::Column::Value)
                    .to_owned(),
            )
            .exec(&self.connection)
            .await?;
        Ok(())
    }
}
