use derive_more::Constructor;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::Set;

use crate::entity::status_list_flag;

#[derive(Debug, Clone, Constructor)]
pub struct Flag {
    connection: DatabaseConnection,
    name: String,
}

impl Flag {
    pub async fn is_set(&self) -> Result<bool, DbErr> {
        let result = status_list_flag::Entity::find()
            .filter(status_list_flag::Column::Name.eq(&self.name))
            .one(&self.connection)
            .await?;

        Ok(result.is_some())
    }

    pub async fn set(&self) -> Result<(), DbErr> {
        let model = status_list_flag::ActiveModel {
            name: Set(self.name.clone()),
        };
        status_list_flag::Entity::insert(model)
            .on_conflict_do_nothing()
            .exec(&self.connection)
            .await?;
        Ok(())
    }
}
