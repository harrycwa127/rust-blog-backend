pub mod post;
pub mod tag;
pub mod post_tag;
pub mod comment;

pub use post::{Entity as Post, Model as PostModel, ActiveModel as PostActiveModel};
pub use tag::{Entity as Tag, Model as TagModel, ActiveModel as TagActiveModel};
pub use post_tag::{Entity as PostTag, Model as PostTagModel, ActiveModel as PostTagActiveModel};
pub use comment::{Entity as Comment, Model as CommentModel, ActiveModel as CommentActiveModel, CommentStatus};

pub mod queries {
    use sea_orm::*;
    use super::*;

    /// 查詢已發布的文章
    pub fn published_posts() -> Select<post::Entity> {
        post::Entity::find()
            .filter(post::Column::IsPublished.eq(true))
            .order_by_desc(post::Column::PublishedAt)
    }

    /// 查詢已審核的留言
    pub fn approved_comments_for_post(post_id: i32) -> Select<comment::Entity> {
        comment::Entity::find()
            .filter(comment::Column::PostId.eq(post_id))
            .filter(comment::Column::Status.eq(CommentStatus::Approved))
            .order_by_asc(comment::Column::CreatedAt)
    }

    /// 查詢標籤及其文章數
    pub fn tags_with_post_count() -> Select<tag::Entity> {
        tag::Entity::find()
            .filter(tag::Column::PostCount.gt(0))
            .order_by_desc(tag::Column::PostCount)
    }
}