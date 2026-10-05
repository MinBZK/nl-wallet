use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "admin_task")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub created_by_user_id: String,
    pub created_by_display_name: String,
    pub task_type: String,
    pub task_parameters: Json,
    pub task_reason: String,
    pub status: String,
    pub reviewed_at: Option<DateTimeWithTimeZone>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_by_display_name: Option<String>,
    pub review_comment: Option<String>,
    pub executed_at: Option<DateTimeWithTimeZone>,
    pub execution_result: Option<String>,
    pub execution_result_details: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
