//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0-rc.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "dimension_label")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub dimension_id: i64,
    pub label_id: i64,
    pub deleted_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::dimension::Entity",
        from = "Column::DimensionId",
        to = "super::dimension::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Dimension,
    #[sea_orm(
        belongs_to = "super::label::Entity",
        from = "Column::LabelId",
        to = "super::label::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Label,
}

impl Related<super::dimension::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dimension.def()
    }
}

impl Related<super::label::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Label.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
