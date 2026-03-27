use crate::dtos::post::*;
use crate::entities::{post, tag, post_tag};
use crate::error::AppError;
use migration::Expr;
use sea_orm::*;
use std::collections::HashMap;
use validator::Validate;
use tracing::{info, error};
use sea_orm::ActiveValue::{Set, NotSet};

pub struct PostService;

impl PostService {
    /// 創建新文章
    pub async fn create_post(
        db: &DatabaseConnection,
        req: CreatePostRequest,
    ) -> Result<PostResponse, AppError> {

        req.validate()
            .map_err(|e| AppError::ValidationError(format!("輸入驗證失敗: {}", e)))?;

        let slug = match req.slug {
            Some(s) if !s.is_empty() => s,
            _ => Self::generate_slug(&req.title),
        };

        if post::Entity::find()
            .filter(post::Column::Slug.eq(&slug))
            .one(db)
            .await?
            .is_some()
        {
            return Err(AppError::ConflictError("該 slug 已經存在".to_string()));
        }

        let excerpt = match req.excerpt {
            Some(e) if !e.is_empty() => Some(e),
            _ => Some(Self::generate_excerpt(&req.content)),
        };

        let is_published = req.is_published.unwrap_or(false);
        let published_at = if is_published {
            Some(chrono::Utc::now())
        } else {
            None
        };

        let mut post_active_model = post::ActiveModel::new();
        post_active_model.id = NotSet;
        post_active_model.title = Set(req.title);
        post_active_model.content = Set(req.content);
        post_active_model.excerpt = Set(excerpt);
        post_active_model.slug = Set(slug);
        post_active_model.is_published = Set(is_published);
        post_active_model.published_at = Set(published_at);

        let post_result = post::Entity::insert(post_active_model)
            .exec_with_returning(db)
            .await
            .map_err(|e| {
                error!("insert post failed: {e:#?}");
                AppError::from(e)
            })?;

        let mut tag_names = Vec::new();
        if let Some(tags) = req.tags {
            for tag_name in tags {
                let name = tag_name.trim();
                if name.is_empty() { continue; }

                Self::create_or_update_tag(db, name, post_result.id).await?;
                tag_names.push(name.to_string());
            }
        }

        Ok(PostResponse {
            id: post_result.id,
            title: post_result.title,
            content: post_result.content,
            excerpt: post_result.excerpt,
            slug: post_result.slug,
            is_published: post_result.is_published,
            view_count: post_result.view_count,
            created_at: post_result.created_at,
            updated_at: post_result.updated_at,
            published_at: post_result.published_at,
            tags: tag_names,
        })
    }

    /// 取得已發布文章列表
    pub async fn get_published_posts(
        db: &DatabaseConnection,
        query: PostListQuery,
    ) -> Result<Vec<PostListResponse>, AppError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10).min(50);
        let offset = (page - 1) * page_size;

        let mut posts_query = post::Entity::find()
            .filter(post::Column::IsPublished.eq(true))
            .order_by_desc(post::Column::PublishedAt);

        if let Some(tag_name) = query.tag {
            let post_ids = post_tag::Entity::find()
                .inner_join(tag::Entity)
                .filter(tag::Column::Name.eq(&tag_name))
                .select_only()
                .column(post_tag::Column::PostId)
                .into_tuple::<i32>()
                .all(db)
                .await?;

            posts_query = posts_query.filter(post::Column::Id.is_in(post_ids));
        }

        let posts = posts_query
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        let post_ids: Vec<i32> = posts.iter().map(|p| p.id).collect();
        let tags_map = Self::get_tags_for_posts(db, &post_ids).await?;

        let response = posts
            .into_iter()
            .map(|post| PostListResponse {
                id: post.id,
                title: post.title,
                excerpt: post.excerpt,
                slug: post.slug,
                view_count: post.view_count,
                published_at: post.published_at,
                tags: tags_map.get(&post.id).cloned().unwrap_or_default(),
            })
            .collect();

        Ok(response)
    }

    /// 生成文章 slug
    fn generate_slug(title: &str) -> String {
        let slug = slug::slugify(title);
        if slug.is_empty() {
            format!("post-{}", chrono::Utc::now().timestamp())
        } else {
            slug
        }
    }

    /// 從內容生成摘要
    fn generate_excerpt(content: &str) -> String {
        // 單純轉純文字＋截斷，避免深層遞迴
        let parser = pulldown_cmark::Parser::new(content);
        let mut html_buf = String::new();
        pulldown_cmark::html::push_html(&mut html_buf, parser);

        let plain_text = html2text::from_read(html_buf.as_bytes(), 200).unwrap_or_default();
        let excerpt = plain_text.trim();
        if excerpt.len() > 200 {
            format!("{}...", &excerpt[..197])
        } else {
            excerpt.to_string()
        }
    }

    /// 創建或更新標籤
    async fn create_or_update_tag(
        db: &DatabaseConnection,
        tag_name: &str,
        post_id: i32,
    ) -> Result<(), AppError> {
        let maybe_tag = tag::Entity::find()
            .filter(tag::Column::Name.eq(tag_name))
            .one(db)
            .await?;

        let tag_model = if let Some(existing) = maybe_tag {
            let mut active: tag::ActiveModel = existing.clone().into();
            active.post_count = Set(existing.post_count + 1);
            tag::Entity::update(active).exec(db).await?
        } else {
            let mut new_tag = tag::ActiveModel::new();
            new_tag.id = NotSet; // 視你的 schema 而定
            new_tag.name = Set(tag_name.to_string());
            new_tag.color = Set(Self::generate_tag_color(tag_name));
            new_tag.post_count = Set(1);
            tag::Entity::insert(new_tag)
                .exec_with_returning(db)
                .await?
        };
        
        let mut pt = post_tag::ActiveModel::new();
        pt.post_id = Set(post_id);
        pt.tag_id = Set(tag_model.id);
        post_tag::Entity::insert(pt).exec(db).await?;

        Ok(())
    }

    /// 批量取得文章的標籤
    async fn get_tags_for_posts(
        db: &DatabaseConnection,
        post_ids: &[i32],
    ) -> Result<HashMap<i32, Vec<String>>, AppError> {
        if post_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let results = post_tag::Entity::find()
            .filter(post_tag::Column::PostId.is_in(post_ids.iter().cloned()))
            .find_also_related(tag::Entity)
            .all(db)
            .await?;

        let mut tags_map: HashMap<i32, Vec<String>> = HashMap::new();
        for (pt, tg) in results {
            if let Some(t) = tg {
                tags_map.entry(pt.post_id).or_default().push(t.name);
            }
        }

        Ok(tags_map)
    }

    /// 為標籤生成隨機顏色
    fn generate_tag_color(tag_name: &str) -> String {
        let colors = [
            "#3498db", "#e74c3c", "#2ecc71", "#f39c12",
            "#9b59b6", "#1abc9c", "#34495e", "#e67e22",
        ];
        let index = tag_name.len() % colors.len();
        colors[index].to_string()
    }

    /// 根據 slug 或 id 取得單篇文章詳情
    pub async fn get_post_by_slug_or_id(
        db: &DatabaseConnection,
        identifier: &str,
    ) -> Result<PostDetailResponse, AppError> {
        // 嘗試解析為數字 ID，否則當作 slug 處理
        let post = if let Ok(id) = identifier.parse::<i32>() {
            post::Entity::find_by_id(id).one(db).await?
        } else {
            post::Entity::find()
                .filter(post::Column::Slug.eq(identifier))
                .one(db)
                .await?
        };

        let post = match post {
            Some(p) => p,
            None => return Err(AppError::NotFound("文章不存在".to_string())),
        };

        // 只有已發布的文章才允許公開查看
        if !post.is_published {
            return Err(AppError::NotFound("文章不存在".to_string()));
        }

        // 取得標籤
        let tags = Self::get_tags_for_post(db, post.id).await?;

        Ok(PostDetailResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            excerpt: post.excerpt,
            slug: post.slug,
            is_published: post.is_published,
            view_count: post.view_count,
            created_at: post.created_at,
            updated_at: post.updated_at,
            published_at: post.published_at,
            tags,
        })
    }

    /// 管理員取得文章詳情（包含草稿）
    pub async fn get_post_for_admin(
        db: &DatabaseConnection,
        post_id: i32,
    ) -> Result<PostDetailResponse, AppError> {
        let post = post::Entity::find_by_id(post_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

        let tags = Self::get_tags_for_post(db, post.id).await?;

        Ok(PostDetailResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            excerpt: post.excerpt,
            slug: post.slug,
            is_published: post.is_published,
            view_count: post.view_count,
            created_at: post.created_at,
            updated_at: post.updated_at,
            published_at: post.published_at,
            tags,
        })
    }

    /// 更新文章
    pub async fn update_post(
        db: &DatabaseConnection,
        post_id: i32,
        req: UpdatePostRequest,
    ) -> Result<PostDetailResponse, AppError> {
        // 檢查文章是否存在
        let existing_post = post::Entity::find_by_id(post_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

        // 保存發布狀態，避免移動後無法訪問
        let was_published = existing_post.is_published;

        let now = chrono::Utc::now();
        let mut updated_post: post::ActiveModel = existing_post.into();

        // 只更新有提供的欄位
        if let Some(title) = req.title {
            updated_post.title = Set(title);
        }

        if let Some(content) = req.content {
            // 如果內容有更新，可能需要重新生成摘要
            let excerpt = req.excerpt.unwrap_or_else(|| {
                Self::generate_excerpt(&content)
            });
            updated_post.content = Set(content);
            updated_post.excerpt = Set(Some(excerpt));
        } else if let Some(excerpt) = req.excerpt {
            updated_post.excerpt = Set(Some(excerpt));
        }

        if let Some(slug) = req.slug {
            // 檢查 slug 是否與其他文章衝突
            if let Some(_conflict) = post::Entity::find()
                .filter(post::Column::Slug.eq(&slug))
                .filter(post::Column::Id.ne(post_id))
                .one(db)
                .await?
            {
                return Err(AppError::ConflictError(format!("Slug '{}' 已被使用", slug)));
            }
            updated_post.slug = Set(slug);
        }

        // 處理發布狀態變更
        if let Some(is_published) = req.is_published {
            updated_post.is_published = Set(is_published);

            // 如果從草稿變為發布，設定發布時間
            if is_published && !was_published {
                updated_post.published_at = Set(Some(now));
                info!("文章 {} 已發布", post_id);
            }
            // 如果從發布變為草稿，清除發布時間
            else if !is_published && was_published {
                updated_post.published_at = Set(None);
                info!("文章 {} 已撤回發布", post_id);
            }
        }

        updated_post.updated_at = Set(now);

        // 開始交易
        let txn = db.begin().await?;

        // 更新文章
        let updated = updated_post.update(&txn).await.map_err(|e| {
            error!("更新文章失敗: {e:#?}");
            AppError::from(e)
        })?;

        // 更新標籤關聯
        if let Some(tags) = req.tags {
            // 刪除現有標籤關聯
            post_tag::Entity::delete_many()
                .filter(post_tag::Column::PostId.eq(post_id))
                .exec(&txn)
                .await?;

            // 重新建立標籤關聯
            for tag_name in tags {
                let name = tag_name.trim();
                if name.is_empty() { continue; }

                Self::create_or_update_tag_txn(&txn, name, post_id).await?;
            }
        }

        txn.commit().await?;

        // 重新取得標籤資料
        let tags = Self::get_tags_for_post(db, updated.id).await?;

        Ok(PostDetailResponse {
            id: updated.id,
            title: updated.title,
            content: updated.content,
            excerpt: updated.excerpt,
            slug: updated.slug,
            is_published: updated.is_published,
            view_count: updated.view_count,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            published_at: updated.published_at,
            tags,
        })
    }

    /// 刪除文章（軟刪除，實際上是設為草稿並隱藏）
    pub async fn delete_post(
        db: &DatabaseConnection,
        post_id: i32,
    ) -> Result<DeletePostResponse, AppError> {
        let post = post::Entity::find_by_id(post_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("文章不存在".to_string()))?;

        // 開始交易
        let txn = db.begin().await?;

        // 刪除標籤關聯
        post_tag::Entity::delete_many()
            .filter(post_tag::Column::PostId.eq(post_id))
            .exec(&txn)
            .await?;

        // 刪除文章
        post::Entity::delete_by_id(post_id)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        info!("文章 {} '{}' 已被刪除", post_id, post.title);

        Ok(DeletePostResponse {
            success: true,
            message: "文章已成功刪除".to_string(),
            deleted_id: post_id,
        })
    }

    /// 增加文章瀏覽次數
    pub async fn increment_view_count(
        db: &DatabaseConnection,
        post_id: i32,
    ) -> Result<(), AppError> {
        post::Entity::update_many()
            .col_expr(post::Column::ViewCount, Expr::add(Expr::col(post::Column::ViewCount), 1))
            .filter(post::Column::Id.eq(post_id))
            .exec(db)
            .await?;

        Ok(())
    }

    /// 取得單篇文章的標籤
    async fn get_tags_for_post(
        db: &DatabaseConnection,
        post_id: i32,
    ) -> Result<Vec<String>, AppError> {
        let results = post_tag::Entity::find()
            .filter(post_tag::Column::PostId.eq(post_id))
            .find_also_related(tag::Entity)
            .all(db)
            .await?;

        let tags = results
            .into_iter()
            .filter_map(|(_, tag)| tag.map(|t| t.name))
            .collect();

        Ok(tags)
    }

    /// 在交易中建立或更新標籤
    async fn create_or_update_tag_txn(
        txn: &DatabaseTransaction,
        tag_name: &str,
        post_id: i32,
    ) -> Result<(), AppError> {
        // 查找或建立標籤
        let tag_entity = match tag::Entity::find()
            .filter(tag::Column::Name.eq(tag_name))
            .one(txn)
            .await?
        {
            Some(existing_tag) => {
                // 更新文章計數
                let mut active_tag: tag::ActiveModel = existing_tag.into();
                active_tag.post_count = Set(active_tag.post_count.unwrap() + 1);
                active_tag.update(txn).await?
            }
            None => {
                // 建立新標籤
                let new_tag = tag::ActiveModel {
                    name: Set(tag_name.to_string()),
                    color: Set(Self::generate_tag_color(tag_name)),
                    post_count: Set(1),
                    ..Default::default()
                };
                new_tag.insert(txn).await?
            }
        };

        // 建立文章-標籤關聯
        let post_tag_relation = post_tag::ActiveModel {
            post_id: Set(post_id),
            tag_id: Set(tag_entity.id),
            ..Default::default()
        };
        post_tag_relation.insert(txn).await?;

        Ok(())
    }
}