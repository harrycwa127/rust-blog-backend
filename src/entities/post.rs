use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize, ToSchema)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    #[schema(example = 1)]
    pub id: i32,

    #[sea_orm(column_type = "String(StringLen::N(255))")]
    #[schema(example = "我的第一篇 Rust 文章")]
    pub title: String,

    #[sea_orm(column_type = "Text")]
    #[schema(example = "今天開始學習 Rust...")]
    pub content: String,

    #[sea_orm(column_type = "String(StringLen::N(500))", nullable)]
    #[schema(example = "這篇文章分享我學習 Rust 的心得...")]
    pub excerpt: Option<String>,

    #[sea_orm(column_type = "String(StringLen::N(255))", unique)]
    #[schema(example = "my-first-rust-article")]
    pub slug: String,

    #[sea_orm(default_value = "false")]
    #[schema(example = true)]
    pub is_published: bool,

    #[sea_orm(default_value = "0")]
    #[schema(example = 42)]
    pub view_count: i32,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub created_at: DateTimeUtc,

    #[schema(value_type = String, example = "2024-01-15T10:30:00Z")]
    pub updated_at: DateTimeUtc,

    #[sea_orm(nullable)]
    #[schema(value_type = Option<String>, example = "2024-01-15T10:30:00Z")]
    pub published_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // 一篇文章有多個留言
    #[sea_orm(has_many = "super::comment::Entity")]
    Comments,

    // 一篇文章經由 pivot post_tag 連到多個 tag
    #[sea_orm(has_many = "super::post_tag::Entity")]
    PostTags,
}

// 經由 pivot 取得 tags
impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::tag::Relation::PostTags.def()
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

            title: NotSet,
            content: NotSet,
            excerpt: NotSet,
            slug: NotSet,

            is_published: Set(false),
            view_count: Set(0),

            // 由這裡幫你預設時間
            created_at: Set(now),
            updated_at: Set(now),

            published_at: NotSet,
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
            if !self.is_published.is_set() {
                self.is_published = Set(false);
            }
            if !self.view_count.is_set() {
                self.view_count = Set(0);
            }
        }

        self.updated_at = Set(now);

        let published = matches!(self.is_published, Set(true));
        let published_at_empty = matches!(self.published_at, NotSet | Set(None));
        if published && published_at_empty {
            self.published_at = Set(Some(now));
        }

        if matches!(self.is_published, Set(false)) {
            self.published_at = Set(None);
        }

        Ok(self)
    }
}