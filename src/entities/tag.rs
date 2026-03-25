use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, ToSchema)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[schema(example = 1)]
    pub id: i32,

    #[sea_orm(column_type = "String(StringLen::N(100))", unique)]
    #[schema(example = "rust")]
    pub name: String,

    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    #[schema(example = "Rust 程式語言相關文章")]
    pub description: Option<String>,

    #[sea_orm(column_type = "String(StringLen::N(7))", default_value = "#6B7280")]
    #[schema(example = "#8B5CF6")]
    pub color: String,

    #[sea_orm(default_value = "0")]
    #[schema(example = 5)]
    pub post_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: DateTimeUtc,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::post_tag::Entity")]
    PostTags,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        super::post::Relation::PostTags.def()
    }
    fn via() -> Option<RelationDef> {
        Some(Relation::PostTags.def().rev())
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        use sea_orm::ActiveValue::{NotSet, Set};
        let now = chrono::Utc::now();
        Self {
            id: NotSet,
            name: NotSet,
            description: NotSet,
            color: Set("#6B7280".to_string()),
            post_count: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        use sea_orm::ActiveValue::{Set};
        let now = chrono::Utc::now();

        if insert {
            if !self.color.is_set() {
                self.color = Set("#6B7280".to_string());
            }
            if !self.post_count.is_set() {
                self.post_count = Set(0);
            }
        }

        self.updated_at = Set(now);

        // color 基本校驗
        if let Set(ref mut c) = self.color {
            if c.len() != 7 || !c.starts_with('#') {
                *c = "#6B7280".to_string();
            }
        }

        Ok(self)
    }
}
