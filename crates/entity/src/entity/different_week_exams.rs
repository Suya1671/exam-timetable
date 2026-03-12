use crate::id::ExamId;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "different_week_exams")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub exam1_id: ExamId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub exam2_id: ExamId,

    #[sea_orm(
        belongs_to,
        relation_enum = "Exam1",
        from = "exam1_id",
        to = "id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    pub exam1: HasOne<super::exams::Entity>,

    #[sea_orm(
        belongs_to,
        relation_enum = "Exam2",
        from = "exam2_id",
        to = "id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    pub exam2: HasOne<super::exams::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
