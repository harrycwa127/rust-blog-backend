use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize, ToSchema)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "comment_status")]
pub enum CommentStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "approved")]
    Approved,
    #[sea_orm(string_value = "rejected")]
    Rejected,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, ToSchema)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[schema(example = 1)]
    pub id: i32,

    #[schema(example = 1)]
    pub post_id: i32,

    #[sea_orm(column_type = "String(StringLen::N(100))")]
    #[schema(example = "讀者小明")]
    pub author_name: String,

    #[sea_orm(column_type = "String(StringLen::N(255))")]
    #[schema(example = "reader@example.com")]
    pub author_email: String,

    #[sea_orm(column_type = "String(StringLen::N(255))", nullable)]
    #[schema(example = "https://example.com")]
    pub author_website: Option<String>,

    #[sea_orm(column_type = "Text")]
    #[schema(example = "很棒的文章！學到很多東西。")]
    pub content: String,

    pub status: CommentStatus,

    #[sea_orm(column_type = "String(StringLen::N(45))", nullable)]
    pub ip_address: Option<String>,

    #[sea_orm(column_type = "Text", nullable)]
    pub user_agent: Option<String>,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: DateTimeUtc,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub updated_at: DateTimeUtc,

    #[sea_orm(nullable)]
    #[schema(value_type = Option<String>, example = "2024-01-15T10:30:00Z")]
    pub approved_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::post::Entity",
        from = "Column::PostId",
        to = "super::post::Column::Id"
    )]
    Post,
}

impl Related<super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        use sea_orm::ActiveValue::{NotSet, Set};
        let now = chrono::Utc::now();
        Self {
            id: NotSet,
            post_id: NotSet,
            author_name: NotSet,
            author_email: NotSet,
            author_website: NotSet,
            content: NotSet,
            status: Set(CommentStatus::Pending),
            ip_address: NotSet,
            user_agent: NotSet,
            created_at: Set(now),
            updated_at: Set(now),
            approved_at: NotSet,
        }
    }

    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        use sea_orm::ActiveValue::{NotSet, Set};
        let now = chrono::Utc::now();

        if insert {
            if !self.created_at.is_set() {
                self.created_at = Set(now);
            }
            if !self.status.is_set() {
                self.status = Set(CommentStatus::Pending);
            }
        }

        // 每次存檔都更新 updated_at
        self.updated_at = Set(now);

        // 若狀態為 approved 且 approved_at 還沒設，補上時間
        let is_approved = matches!(self.status, Set(CommentStatus::Approved));
        let approved_at_empty = matches!(self.approved_at, NotSet | Set(None));
        if is_approved && approved_at_empty {
            self.approved_at = Set(Some(now));
        }

        Ok(self)
    }
}
