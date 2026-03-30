use crate::dtos::post::*;
use crate::entities::{post, tag, post_tag, comment, CommentStatus};
use crate::error::AppError;
use migration::Expr;
use sea_orm::*;
use std::collections::HashMap;
use validator::Validate;
use tracing::{info, error};
use sea_orm::ActiveValue::{Set, NotSet};
use std::fmt;
use sea_orm::sea_query::Query;

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

        let txn = db.begin().await?;

        let mut post_active_model = post::ActiveModel::new();
        post_active_model.title = Set(req.title);
        post_active_model.content = Set(req.content);
        post_active_model.excerpt = Set(excerpt);
        post_active_model.slug = Set(slug);
        post_active_model.is_published = Set(is_published);
        post_active_model.published_at = Set(published_at);

        let post_result = post::Entity::insert(post_active_model)
            .exec_with_returning(&txn)
            .await
            .map_err(|e| {
                error!("insert post failed: {e:#?}");
                AppError::from(e)
            })?;

        let mut tag_names: Vec<String> = Vec::new();
        if let Some(tags) = req.tags {
            for tag_name in tags {
                let name = tag_name.trim();
                if name.is_empty() { continue; }
                Self::create_or_update_tag_txn(&txn, name, post_result.id).await?;
                tag_names.push(name.to_string());
            }
        }

        Self::sync_tag_counts_for_post(&txn, vec![], tag_names.clone()).await?;

        txn.commit().await?;

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

        if let Some(tags) = req.tags {

            let old_tags: Vec<String> = post_tag::Entity::find()
                .filter(post_tag::Column::PostId.eq(post_id))
                .find_also_related(tag::Entity)
                .all(&txn)
                .await?
                .into_iter()
                .filter_map(|(_, t)| t.map(|x| x.name))
                .collect();

            post_tag::Entity::delete_many()
                .filter(post_tag::Column::PostId.eq(post_id))
                .exec(&txn)
                .await?;

            let mut new_tags: Vec<String> = Vec::new();
            for tag_name in tags {
                let name = tag_name.trim();
                if name.is_empty() { continue; }
                Self::create_or_update_tag_txn(&txn, name, post_id).await?;
                new_tags.push(name.to_string());
            }

            Self::sync_tag_counts_for_post(&txn, old_tags, new_tags).await?;
        }

        // 更新文章
        let updated = updated_post.update(&txn).await.map_err(|e| {
            error!("更新文章失敗: {e:#?}");
            AppError::from(e)
        })?;

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

        let txn = db.begin().await?;

        let old_tags: Vec<String> = post_tag::Entity::find()
            .filter(post_tag::Column::PostId.eq(post_id))
            .find_also_related(tag::Entity)
            .all(&txn)
            .await?
            .into_iter()
            .filter_map(|(_, t)| t.map(|x| x.name))
            .collect();

        // ❷ 刪除關聯與文章
        post_tag::Entity::delete_many()
            .filter(post_tag::Column::PostId.eq(post_id))
            .exec(&txn)
            .await?;

        post::Entity::delete_by_id(post_id).exec(&txn).await?;

        // ❸ 同步計數：old=old_tags，new=[]
        Self::sync_tag_counts_for_post(&txn, old_tags, vec![]).await?;

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

        let post_tag_relation = post_tag::ActiveModel {
            post_id: Set(post_id),
            tag_id: Set(tag_entity.id),
            ..Default::default()
        };
        post_tag_relation.insert(txn).await?;

        Ok(())
    }

    async fn sync_tag_counts_for_post(
        txn: &DatabaseTransaction,
        old_tags: Vec<String>,
        new_tags: Vec<String>,
    ) -> Result<(), AppError> {
        let removed_tags: Vec<_> = old_tags.iter()
            .filter(|tag| !new_tags.contains(tag))
            .collect();

        // 找出新增的標籤
        let added_tags: Vec<_> = new_tags.iter()
            .filter(|tag| !old_tags.contains(tag))
            .collect();

        // 更新所有受影響標籤的計數
        let mut affected_tag_ids = Vec::new();

        // 取得被移除標籤的 ID
        if !removed_tags.is_empty() {
            let removed_tag_entities = tag::Entity::find()
                .filter(tag::Column::Name.is_in(removed_tags))
                .all(txn)
                .await?;
            affected_tag_ids.extend(removed_tag_entities.iter().map(|t| t.id));
        }

        // 取得新增標籤的 ID
        if !added_tags.is_empty() {
            let added_tag_entities = tag::Entity::find()
                .filter(tag::Column::Name.is_in(added_tags))
                .all(txn)
                .await?;
            affected_tag_ids.extend(added_tag_entities.iter().map(|t| t.id));
        }

        // 更新所有受影響標籤的文章計數
        for tag_id in affected_tag_ids {
            let count = post_tag::Entity::find()
                .filter(post_tag::Column::TagId.eq(tag_id))
                .inner_join(post::Entity)
                .filter(post::Column::IsPublished.eq(true))
                .count(txn)
                .await?;

            tag::Entity::update_many()
                .col_expr(tag::Column::PostCount, Expr::value(count as i32))
                .filter(tag::Column::Id.eq(tag_id))
                .exec(txn)
                .await?;
        }

        Ok(())
    }

    pub async fn search_posts(
        db: &DatabaseConnection,
        query: PostSearchQuery,
        is_admin: bool,
    ) -> Result<PostSearchResponse, AppError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(10).min(50); // 限制最大每頁數量
        let offset = (page - 1) * page_size;

        // 建立基本查詢
        let mut select = post::Entity::find();

        // 狀態篩選
        match query.status.as_deref() {
            Some("published") => {
                select = select.filter(post::Column::IsPublished.eq(true));
            },
            Some("draft") => {
                if is_admin {
                    select = select.filter(post::Column::IsPublished.eq(false));
                } else {
                    return Err(AppError::Forbidden("無權限查看草稿".to_string()));
                }
            },
            Some("all") => {
                if !is_admin {
                    select = select.filter(post::Column::IsPublished.eq(true));
                }
            },
            _ => {
                if !is_admin {
                    select = select.filter(post::Column::IsPublished.eq(true));
                }
            }
        }

        // 關鍵字搜尋
        let applied_keyword = if let Some(ref keyword) = query.q {
            let search_term = format!("%{}%", keyword.trim());
            select = select.filter(
                Condition::any()
                    .add(post::Column::Title.like(&search_term))
                    .add(post::Column::Content.like(&search_term))
                    .add(post::Column::Excerpt.like(&search_term))
            );
            Some(keyword.clone())
        } else {
            None
        };

        // 標籤篩選
        let applied_tags = if let Some(ref tags_str) = query.tags {
            let tag_names: Vec<&str> = tags_str.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if !tag_names.is_empty() {
                // 使用 EXISTS 子查詢來篩選包含指定標籤的文章
                for tag_name in &tag_names {
                    select = select.filter(
                        Expr::exists(
                            Query::select()
                                .column(post_tag::Column::PostId)
                                .from(post_tag::Entity)
                                .inner_join(
                                    tag::Entity,
                                    Expr::col((tag::Entity, tag::Column::Id))
                                        .equals((post_tag::Entity, post_tag::Column::TagId))
                                )
                                .and_where(
                                    Expr::col((post_tag::Entity, post_tag::Column::PostId))
                                        .equals((post::Entity, post::Column::Id))
                                )
                                .and_where(
                                    Expr::col((tag::Entity, tag::Column::Name))
                                        .eq(*tag_name)
                                )
                                .take()
                        )
                    );
                }
                tag_names.into_iter().map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // 日期範圍篩選
        let (date_start, date_end) = if let (Some(from), Some(to)) = (&query.from_date, &query.to_date) {
            if let (Ok(start_date), Ok(end_date)) = (
                chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d"),
                chrono::NaiveDate::parse_from_str(to, "%Y-%m-%d")
            ) {
                let start_datetime = start_date.and_hms_opt(0, 0, 0)
                    .ok_or_else(|| AppError::BadRequest("無效的開始日期".to_string()))?
                    .and_utc();
                let end_datetime = end_date.and_hms_opt(23, 59, 59)
                    .ok_or_else(|| AppError::BadRequest("無效的結束日期".to_string()))?
                    .and_utc();

                select = select.filter(
                    post::Column::CreatedAt.between(start_datetime, end_datetime)
                );

                (Some(from.clone()), Some(to.clone()))
            } else {
                return Err(AppError::BadRequest("日期格式錯誤，請使用 YYYY-MM-DD 格式".to_string()));
            }
        } else {
            (None, None)
        };

        // 排序
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("desc");

        select = match (sort_by, sort_order) {
            ("title", "desc") => select.order_by_desc(post::Column::Title),
            ("title", _) => select.order_by_asc(post::Column::Title),
            ("updated_at", "asc") => select.order_by_asc(post::Column::UpdatedAt),
            ("updated_at", _) => select.order_by_desc(post::Column::UpdatedAt),
            ("view_count", "asc") => select.order_by_asc(post::Column::ViewCount),
            ("view_count", _) => select.order_by_desc(post::Column::ViewCount),
            ("created_at", "asc") => select.order_by_asc(post::Column::CreatedAt),
            _ => select.order_by_desc(post::Column::CreatedAt),
        };

        // 計算總數
        let total_count = select.clone().count(db).await?;
        let total_pages = (total_count + page_size - 1) / page_size;

        // 執行分頁查詢
        let posts = select
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        // 取得文章的標籤和留言數
        let mut post_responses = Vec::new();
        for post in posts {
            let tags = Self::get_tags_for_post(db, post.id).await?
                .into_iter()
                .map(|tag_name| TagSummaryResponse {
                    id: 0, 
                    name: tag_name.clone(),
                    color: Self::generate_tag_color(&tag_name),
                })
                .collect();

            // 計算留言數（只計算已審核的）
            let comment_count = comment::Entity::find()
                .filter(comment::Column::PostId.eq(post.id))
                .filter(comment::Column::Status.eq(CommentStatus::Approved))
                .count(db)
                .await? as i32;

            post_responses.push(PostSummaryResponse {
                id: post.id,
                title: post.title,
                excerpt: post.excerpt,
                slug: post.slug,
                is_published: post.is_published,
                view_count: post.view_count,
                created_at: post.created_at,
                updated_at: post.updated_at,
                published_at: post.published_at,
                tags,
                comment_count,
            });
        }

        // 產生搜尋摘要
        let search_summary = Self::generate_search_summary(
            total_count,
            &applied_keyword,
            &applied_tags,
            &date_start,
            &date_end,
        );

        Ok(PostSearchResponse {
            total_count: total_count as i64,
            total_pages,
            current_page: page,
            page_size,
            posts: post_responses,
            search_summary,
            filters_applied: SearchFiltersApplied {
                keyword: applied_keyword,
                tags: applied_tags,
                status: query.status,
                date_range_start: date_start,
                date_range_end: date_end,
            },
        })
    }

    /// 取得熱門搜尋與建議
    pub async fn get_popular_search_data(
        db: &DatabaseConnection,
    ) -> Result<PopularSearchResponse, AppError> {
        // 取得熱門標籤（按文章數排序）
        let popular_tags = tag::Entity::find()
            .filter(tag::Column::PostCount.gt(0))
            .order_by_desc(tag::Column::PostCount)
            .limit(10)
            .all(db)
            .await?
            .into_iter()
            .map(|tag| tag.name)
            .collect();

        // 取得最近發布的文章
        let recent_posts = post::Entity::find()
            .filter(post::Column::IsPublished.eq(true))
            .order_by_desc(post::Column::PublishedAt)
            .limit(5)
            .all(db)
            .await?;

        let mut recent_post_responses = Vec::new();
        for post in recent_posts {
            let tags = Self::get_tags_for_post(db, post.id).await?
                .into_iter()
                .map(|tag_name| TagSummaryResponse {
                    id: 0,
                    name: tag_name.clone(),
                    color: Self::generate_tag_color(&tag_name),
                })
                .collect();

            let comment_count = comment::Entity::find()
                .filter(comment::Column::PostId.eq(post.id))
                .filter(comment::Column::Status.eq(CommentStatus::Approved))
                .count(db)
                .await? as i32;

            recent_post_responses.push(PostSummaryResponse {
                id: post.id,
                title: post.title,
                excerpt: post.excerpt,
                slug: post.slug,
                is_published: post.is_published,
                view_count: post.view_count,
                created_at: post.created_at,
                updated_at: post.updated_at,
                published_at: post.published_at,
                tags,
                comment_count,
            });
        }

        // 搜尋建議（基於文章標題的常見關鍵字）
        let search_suggestions = vec![
            "Rust".to_string(),
            "程式設計".to_string(),
            "Web 開發".to_string(),
            "後端".to_string(),
            "教學".to_string(),
        ];

        // 統計資料
        let total_posts = post::Entity::find().count(db).await? as i64;
        let total_published = post::Entity::find()
            .filter(post::Column::IsPublished.eq(true))
            .count(db)
            .await? as i64;

        Ok(PopularSearchResponse {
            popular_tags,
            recent_posts: recent_post_responses,
            search_suggestions,
            total_posts,
            total_published,
        })
    }

    /// 產生搜尋摘要文字
    fn generate_search_summary(
        total_count: u64,
        keyword: &Option<String>,
        tags: &[String],
        date_start: &Option<String>,
        date_end: &Option<String>,
    ) -> String {
        let mut parts = Vec::new();

        parts.push(format!("找到 {} 篇文章", total_count));

        if let Some(kw) = keyword {
            parts.push(format!("包含關鍵字 '{}'", kw));
        }

        if !tags.is_empty() {
            if tags.len() == 1 {
                parts.push(format!("標籤為 '{}'", tags[0]));
            } else {
                parts.push(format!("標籤包含 {}", tags.join(", ")));
            }
        }

        if let (Some(start), Some(end)) = (date_start, date_end) {
            parts.push(format!("發布日期在 {} 到 {} 之間", start, end));
        }

        parts.join("，")
    }
}