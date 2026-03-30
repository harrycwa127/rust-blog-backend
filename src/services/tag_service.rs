use sea_orm::*;
use sea_orm::prelude::Expr;
use tracing::{info, error};

use crate::{
    dtos::{
        CreateTagRequest, UpdateTagRequest, TagResponse, TagWithPostsResponse,
        DeleteTagResponse, TagListQuery, TagSuggestionResponse, PostListResponse,
    },
    entities::{tag, post, post_tag},
    error::AppError,
    utils::validation::{generate_random_color, sanitize_tag_name},
};

pub struct TagService;

impl TagService {
    pub async fn get_all_tags(
        db: &DatabaseConnection,
    ) -> Result<Vec<TagResponse>, AppError> {
        let tags = tag::Entity::find()
            .all(db)
            .await?;

        let tag_responses = tags.into_iter().map(|tag| TagResponse {
            id: tag.id,
            name: tag.name,
            description: tag.description,
            color: tag.color,
            post_count: tag.post_count,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }).collect();

        Ok(tag_responses)
    }

    /// 取得標籤列表
    pub async fn get_tags(
        db: &DatabaseConnection,
        query: TagListQuery,
    ) -> Result<Vec<TagResponse>, AppError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20).min(100); // 限制最大 100 筆
        let offset = (page - 1) * page_size;

        let mut select = tag::Entity::find();

        // 搜尋功能
        if let Some(search) = &query.search {
            let search_term = format!("%{}%", search.trim());
            select = select.filter(
                Condition::any()
                    .add(tag::Column::Name.like(&search_term))
                    .add(tag::Column::Description.like(&search_term))
            );
        }

        // 是否包含空標籤（沒有文章的標籤）
        if !query.include_empty.unwrap_or(true) {
            select = select.filter(tag::Column::PostCount.gt(0));
        }

        // 排序
        let sort_by = query.sort_by.as_deref().unwrap_or("name");
        let sort_order = query.sort_order.as_deref().unwrap_or("asc");

        select = match (sort_by, sort_order) {
            ("name", "desc") => select.order_by_desc(tag::Column::Name),
            ("name", _) => select.order_by_asc(tag::Column::Name),
            ("post_count", "asc") => select.order_by_asc(tag::Column::PostCount),
            ("post_count", _) => select.order_by_desc(tag::Column::PostCount),
            ("created_at", "desc") => select.order_by_desc(tag::Column::CreatedAt),
            ("created_at", _) => select.order_by_asc(tag::Column::CreatedAt),
            _ => select.order_by_asc(tag::Column::Name),
        };

        let tags = select
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        let tag_responses = tags.into_iter().map(|tag| TagResponse {
            id: tag.id,
            name: tag.name,
            description: tag.description,
            color: tag.color,
            post_count: tag.post_count,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }).collect();

        Ok(tag_responses)
    }

    /// 根據 ID 取得單一標籤
    pub async fn get_tag_by_id(
        db: &DatabaseConnection,
        tag_id: i32,
    ) -> Result<TagResponse, AppError> {
        let tag = tag::Entity::find_by_id(tag_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("標籤不存在".to_string()))?;

        Ok(TagResponse {
            id: tag.id,
            name: tag.name,
            description: tag.description,
            color: tag.color,
            post_count: tag.post_count,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        })
    }

    /// 根據名稱取得標籤及其文章
    pub async fn get_tag_with_posts(
        db: &DatabaseConnection,
        tag_name: &str,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<TagWithPostsResponse, AppError> {
        let tag = tag::Entity::find()
            .filter(tag::Column::Name.eq(tag_name))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("標籤 '{}' 不存在", tag_name)))?;

        // 分頁設定
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10).min(50);
        let offset = (page - 1) * page_size;

        // 查詢該標籤的已發布文章
        let posts = post::Entity::find()
            .filter(post::Column::IsPublished.eq(true))
            .join(JoinType::InnerJoin, post_tag::Relation::Post.def())
            .filter(post_tag::Column::TagId.eq(tag.id))
            .order_by_desc(post::Column::PublishedAt)
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        let post_responses = posts.into_iter().map(|post| PostListResponse {
            id: post.id,
            title: post.title,
            excerpt: post.excerpt,
            slug: post.slug,
            view_count: post.view_count,
            published_at: post.published_at,
            tags: vec![], // 這裡可以後續優化，避免 N+1 查詢
        }).collect();

        Ok(TagWithPostsResponse {
            id: tag.id,
            name: tag.name,
            description: tag.description,
            color: tag.color,
            post_count: tag.post_count,
            created_at: tag.created_at,
            posts: post_responses,
        })
    }

    /// 建立新標籤
    pub async fn create_tag(
        db: &DatabaseConnection,
        req: CreateTagRequest,
    ) -> Result<TagResponse, AppError> {
        let sanitized_name = sanitize_tag_name(&req.name);

        if sanitized_name.is_empty() {
            return Err(AppError::BadRequest("標籤名稱不能為空".to_string()));
        }

        // 檢查標籤名稱是否已存在
        let existing = tag::Entity::find()
            .filter(tag::Column::Name.eq(&sanitized_name))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest(format!("標籤 '{}' 已存在", sanitized_name)));
        }

        let color = req.color.unwrap_or_else(generate_random_color);

        let tag_active_model = tag::ActiveModel {
            name: Set(sanitized_name),
            description: Set(req.description),
            color: Set(color),
            ..Default::default()
        };

        let tag = tag_active_model.insert(db).await.map_err(|e| {
            error!("建立標籤失敗: {e:#?}");
            AppError::from(e)
        })?;

        info!("標籤 '{}' 建立成功，ID: {}", tag.name, tag.id);

        Ok(TagResponse {
            id: tag.id,
            name: tag.name,
            description: tag.description,
            color: tag.color,
            post_count: tag.post_count,
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        })
    }

    /// 更新標籤
    pub async fn update_tag(
        db: &DatabaseConnection,
        tag_id: i32,
        req: UpdateTagRequest,
    ) -> Result<TagResponse, AppError> {
        let tag = tag::Entity::find_by_id(tag_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("標籤不存在".to_string()))?;

        let mut updated_tag: tag::ActiveModel = tag.into();

        if let Some(name) = req.name {
            let sanitized_name = sanitize_tag_name(&name);

            if sanitized_name.is_empty() {
                return Err(AppError::BadRequest("標籤名稱不能為空".to_string()));
            }

            // 檢查新名稱是否與其他標籤衝突
            if sanitized_name != *updated_tag.name.as_ref() {
                let existing = tag::Entity::find()
                    .filter(tag::Column::Name.eq(&sanitized_name))
                    .filter(tag::Column::Id.ne(tag_id))
                    .one(db)
                    .await?;

                if existing.is_some() {
                    return Err(AppError::BadRequest(format!("標籤 '{}' 已存在", sanitized_name)));
                }
            }

            updated_tag.name = Set(sanitized_name);
        }

        if let Some(description) = req.description {
            updated_tag.description = Set(Some(description));
        }

        if let Some(color) = req.color {
            updated_tag.color = Set(color);
        }

        let updated = updated_tag.update(db).await.map_err(|e| {
            error!("更新標籤失敗: {e:#?}");
            AppError::from(e)
        })?;

        info!("標籤 {} '{}' 更新成功", tag_id, updated.name);

        Ok(TagResponse {
            id: updated.id,
            name: updated.name,
            description: updated.description,
            color: updated.color,
            post_count: updated.post_count,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        })
    }

    /// 刪除標籤
    pub async fn delete_tag(
        db: &DatabaseConnection,
        tag_id: i32,
    ) -> Result<DeleteTagResponse, AppError> {
        let tag = tag::Entity::find_by_id(tag_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("標籤不存在".to_string()))?;

        // 計算受影響的文章數量
        let affected_posts = post_tag::Entity::find()
            .filter(post_tag::Column::TagId.eq(tag_id))
            .count(db)
            .await?;

        // 開始交易
        let txn = db.begin().await?;

        // 刪除所有相關的文章-標籤關聯
        post_tag::Entity::delete_many()
            .filter(post_tag::Column::TagId.eq(tag_id))
            .exec(&txn)
            .await?;

        // 刪除標籤
        tag::Entity::delete_by_id(tag_id)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        info!("標籤 {} '{}' 已刪除，影響 {} 篇文章", tag_id, tag.name, affected_posts);

        Ok(DeleteTagResponse {
            success: true,
            message: format!("標籤 '{}' 已成功刪除", tag.name),
            deleted_id: tag_id,
            affected_posts: affected_posts as i32,
        })
    }

    /// 取得標籤建議
    pub async fn get_tag_suggestions(
        db: &DatabaseConnection,
        query: Option<String>,
        limit: Option<u64>,
    ) -> Result<TagSuggestionResponse, AppError> {
        let limit = limit.unwrap_or(10).min(20);

        let mut suggestions = Vec::new();

        // 如果有查詢字串，搜尋相似的標籤
        if let Some(q) = &query {
            if !q.trim().is_empty() {
                let search_term = format!("%{}%", q.trim());
                let similar_tags = tag::Entity::find()
                    .filter(tag::Column::Name.like(&search_term))
                    .order_by_desc(tag::Column::PostCount)
                    .limit(limit)
                    .all(db)
                    .await?;

                suggestions = similar_tags.into_iter().map(|tag| tag.name).collect();
            }
        }

        // 取得熱門標籤
        let popular_tags = tag::Entity::find()
            .filter(tag::Column::PostCount.gt(0))
            .order_by_desc(tag::Column::PostCount)
            .limit(10)
            .all(db)
            .await?;

        let popular_tag_names = popular_tags.into_iter().map(|tag| tag.name).collect();

        // 取得總標籤數
        let total_tags = tag::Entity::find().count(db).await? as i32;

        Ok(TagSuggestionResponse {
            suggestions,
            total_tags,
            popular_tags: popular_tag_names,
        })
    }

    /// 更新標籤的文章計數（內部使用）
    pub async fn update_tag_post_count(
        db: &DatabaseConnection,
        tag_id: i32,
    ) -> Result<(), AppError> {
        let count = post_tag::Entity::find()
            .filter(post_tag::Column::TagId.eq(tag_id))
            .inner_join(post::Entity)
            .filter(post::Column::IsPublished.eq(true))
            .count(db)
            .await?;

        tag::Entity::update_many()
            .col_expr(tag::Column::PostCount, Expr::value(count as i32))
            .filter(tag::Column::Id.eq(tag_id))
            .exec(db)
            .await?;

        Ok(())
    }
}