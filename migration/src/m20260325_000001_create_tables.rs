use sea_orm_migration::prelude::*;
use crate::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // posts
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::Title).string_len(255).not_null())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Excerpt).string_len(500))
                    .col(
                        ColumnDef::new(Posts::Slug)
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Posts::IsPublished)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Posts::ViewCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Posts::PublishedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // tags
        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tags::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Tags::Name)
                            .string_len(100)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Tags::Description).string_len(255))
                    .col(
                        ColumnDef::new(Tags::Color)
                            .string_len(7)
                            .not_null()
                            .default("#6B7280"),
                    )
                    .col(
                        ColumnDef::new(Tags::PostCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Tags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Tags::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // post_tags
        manager
            .create_table(
                Table::create()
                    .table(PostTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PostTags::PostId).integer().not_null())
                    .col(ColumnDef::new(PostTags::TagId).integer().not_null())
                    .col(
                        ColumnDef::new(PostTags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(Index::create().col(PostTags::PostId).col(PostTags::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_tags_post_id")
                            .from(PostTags::Table, PostTags::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_tags_tag_id")
                            .from(PostTags::Table, PostTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // enum comment_status
        manager
            .create_type(
                Type::create()
                    .as_enum(CommentStatus::Enum)
                    .values([
                        CommentStatus::Pending,
                        CommentStatus::Approved,
                        CommentStatus::Rejected,
                    ])
                    .to_owned(),
            )
            .await?;

        // comments
        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Comments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Comments::PostId).integer().not_null())
                    .col(ColumnDef::new(Comments::AuthorName).string_len(100).not_null())
                    .col(ColumnDef::new(Comments::AuthorEmail).string_len(255).not_null())
                    .col(ColumnDef::new(Comments::AuthorWebsite).string_len(255))
                    .col(ColumnDef::new(Comments::Content).text().not_null())
                    .col(
                        ColumnDef::new(Comments::Status)
                            .enumeration(
                                CommentStatus::Enum,
                                [
                                    CommentStatus::Pending,
                                    CommentStatus::Approved,
                                    CommentStatus::Rejected,
                                ],
                            )
                            .not_null()
                            // 明確 cast 成 PostgreSQL enum
                            .default(Expr::cust("'pending'::comment_status")),
                    )
                    .col(ColumnDef::new(Comments::IpAddress).string_len(45))
                    .col(ColumnDef::new(Comments::UserAgent).text())
                    .col(
                        ColumnDef::new(Comments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Comments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Comments::ApprovedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_post_id")
                            .from(Comments::Table, Comments::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_posts_published")
                    .table(Posts::Table)
                    .col(Posts::IsPublished)
                    .col(Posts::PublishedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_posts_slug")
                    .table(Posts::Table)
                    .col(Posts::Slug)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_comments_post_status")
                    .table(Comments::Table)
                    .col(Comments::PostId)
                    .col(Comments::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // drop indexes
        manager
            .drop_index(Index::drop().name("idx_comments_post_status").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_posts_slug").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_posts_published").to_owned())
            .await?;

        // drop tables (FK 依賴順序)
        manager
            .drop_table(Table::drop().table(Comments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PostTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        // drop enum type
        manager
            .drop_type(Type::drop().name(CommentStatus::Enum).to_owned())
            .await?;

        Ok(())
    }
}

// ===== Idens =====

#[derive(Iden)]
enum Posts {
    Table,
    Id,
    Title,
    Content,
    Excerpt,
    Slug,
    IsPublished,
    ViewCount,
    CreatedAt,
    UpdatedAt,
    PublishedAt,
}

#[derive(Iden)]
enum Tags {
    Table,
    Id,
    Name,
    Description,
    Color,
    PostCount,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PostTags {
    Table,
    PostId,
    TagId,
    CreatedAt,
}

#[derive(Iden)]
enum Comments {
    Table,
    Id,
    PostId,
    AuthorName,
    AuthorEmail,
    AuthorWebsite,
    Content,
    Status,
    IpAddress,
    UserAgent,
    CreatedAt,
    UpdatedAt,
    ApprovedAt,
}

#[derive(Iden)]
enum CommentStatus {
    #[iden = "comment_status"] // PostgreSQL enum type name
    Enum,
    #[iden = "pending"]
    Pending,
    #[iden = "approved"]
    Approved,
    #[iden = "rejected"]
    Rejected,
}