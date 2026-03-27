use sea_orm::{ActiveModelBehavior, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait};
use tracing::{info, warn};
use validator::Validate;
use std::fmt;

use crate::{
    dtos::{
        CreateCommentRequest, CommentResponse, CommentListQuery,
        UpdateCommentStatusRequest, CommentModerationResponse,
    },
    entities::{comment, post, comment::CommentStatus},
    error::AppError,
};

// Implement Display for CommentStatus
impl fmt::Display for CommentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommentStatus::Pending => write!(f, "pending"),
            CommentStatus::Approved => write!(f, "approved"),
            CommentStatus::Rejected => write!(f, "rejected"),
        }
    }
}

pub struct CommentService;

impl CommentService {
    /// 為文章建立留言
    pub async fn create_comment(
        db: &DatabaseConnection,
        post_id: i32,
        req: CreateCommentRequest,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<CommentResponse, AppError> {
        // 驗證輸入
        req.validate()
            .map_err(|e| AppError::ValidationError(format!("留言資料驗證失敗: {}", e)))?;

        let txn = db.begin().await?;

        // 檢查文章是否存在且已發布
        let post = post::Entity::find_by_id(post_id)
            .filter(post::Column::IsPublished.eq(true))
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("文章不存在或未發布".to_string()))?;

        // 清理內容和偵測垃圾留言
        let cleaned_content = Self::sanitize_content(&req.content);
        let is_spam = Self::detect_spam(&cleaned_content, &req.author_name);

        // 建立留言
        let mut new_comment = comment::ActiveModel::new();
        new_comment.post_id = Set(post_id);
        new_comment.author_name = Set(req.author_name.trim().to_string());
        new_comment.author_email = Set(req.author_email.trim().to_lowercase());
        new_comment.author_website = Set(req.author_website.as_ref().map(|url| url.trim().to_string()));
        new_comment.content = Set(cleaned_content);
        new_comment.status = Set(if is_spam {
            CommentStatus::Rejected
        } else {
            CommentStatus::Pending
        });
        new_comment.ip_address = Set(ip_address);
        new_comment.user_agent = Set(user_agent);

        let comment_model = comment::Entity::insert(new_comment)
            .exec_with_returning(&txn)
            .await?;

        txn.commit().await?;

        info!("新留言建立成功: post_id={}, comment_id={}, is_spam={}", 
              post_id, comment_model.id, is_spam);

        Ok(CommentResponse {
            id: comment_model.id,
            post_id: comment_model.post_id,
            author_name: comment_model.author_name,
            author_website: comment_model.author_website,
            content: comment_model.content,
            status: comment_model.status.to_string(),
            created_at: comment_model.created_at,
        })
    }

    /// 取得文章的留言列表（僅顯示已審核的）
    pub async fn get_comments_for_post(
        db: &DatabaseConnection,
        post_id: i32,
        query: CommentListQuery,
    ) -> Result<Vec<CommentResponse>, AppError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20).min(50);
        let offset = (page - 1) * page_size;

        // 只顯示已審核的留言
        let mut select = comment::Entity::find()
            .filter(comment::Column::PostId.eq(post_id))
            .filter(comment::Column::Status.eq(CommentStatus::Approved));

        // 排序：預設由舊到新
        match query.sort_order.as_deref().unwrap_or("asc") {
            "desc" => select = select.order_by_desc(comment::Column::CreatedAt),
            _ => select = select.order_by_asc(comment::Column::CreatedAt),
        }

        let comments = select
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        let responses = comments.into_iter().map(|comment| CommentResponse {
            id: comment.id,
            post_id: comment.post_id,
            author_name: comment.author_name,
            author_website: comment.author_website,
            content: comment.content,
            status: comment.status.to_string(),
            created_at: comment.created_at,
        }).collect();

        Ok(responses)
    }

    /// 取得留言列表（管理員用）
    pub async fn get_comments_for_admin(
        db: &DatabaseConnection,
        query: CommentListQuery,
    ) -> Result<Vec<CommentModerationResponse>, AppError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut select = comment::Entity::find()
            .find_also_related(post::Entity);

        // 狀態篩選
        match query.status.as_deref() {
            Some("pending") => select = select.filter(comment::Column::Status.eq(CommentStatus::Pending)),
            Some("approved") => select = select.filter(comment::Column::Status.eq(CommentStatus::Approved)),
            Some("rejected") => select = select.filter(comment::Column::Status.eq(CommentStatus::Rejected)),
            Some("all") | _ => {}, // 管理員可以看到所有狀態
        }

        // 排序：預設最新的在前面
        match query.sort_order.as_deref().unwrap_or("desc") {
            "asc" => select = select.order_by_asc(comment::Column::CreatedAt),
            _ => select = select.order_by_desc(comment::Column::CreatedAt),
        }

        let results = select
            .offset(offset)
            .limit(page_size)
            .all(db)
            .await?;

        let responses = results.into_iter().map(|(comment, post_opt)| {
            let post_title = match &post_opt {
                Some(post) => post.title.clone(),
                None => String::new(),
            };
            CommentModerationResponse {
                id: comment.id,
                post_id: comment.post_id,
                author_name: comment.author_name,
                author_email: comment.author_email,
                author_website: comment.author_website,
                content: comment.content,
                status: comment.status.to_string(),
                ip_address: comment.ip_address,
                created_at: comment.created_at,
                post_title: post_title,
            }
        }).collect();

        Ok(responses)
    }

    /// 更新留言狀態（管理員審核）
    pub async fn update_comment_status(
        db: &DatabaseConnection,
        comment_id: i32,
        req: UpdateCommentStatusRequest,
    ) -> Result<CommentResponse, AppError> {
        let txn = db.begin().await?;

        let comment = comment::Entity::find_by_id(comment_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("留言不存在".to_string()))?;

        let new_status = match req.status.as_str() {
            "approved" => CommentStatus::Approved,
            "rejected" => CommentStatus::Rejected,
            "pending" => CommentStatus::Pending,
            _ => return Err(AppError::BadRequest("無效的狀態值".to_string())),
        };

        let mut active_comment: comment::ActiveModel = comment.clone().into();
        active_comment.status = Set(new_status.clone());

        let updated_comment = comment::Entity::update(active_comment)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        info!("留言狀態更新: comment_id={}, new_status={:?}, reason={:?}", 
              comment_id, new_status, req.reason);

        Ok(CommentResponse {
            id: updated_comment.id,
            post_id: updated_comment.post_id,
            author_name: updated_comment.author_name,
            author_website: updated_comment.author_website,
            content: updated_comment.content,
            status: updated_comment.status.to_string(),
            created_at: updated_comment.created_at,
        })
    }

    /// 刪除留言（管理員用）
    pub async fn delete_comment(
        db: &DatabaseConnection,
        comment_id: i32,
    ) -> Result<(), AppError> {
        let txn = db.begin().await?;

        let comment = comment::Entity::find_by_id(comment_id)
            .one(&txn)
            .await?
            .ok_or_else(|| AppError::NotFound("留言不存在".to_string()))?;

        // 刪除留言
        comment::Entity::delete_by_id(comment_id)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        info!("留言刪除成功: comment_id={}", comment_id);
        Ok(())
    }

    /// 垃圾留言偵測
    fn detect_spam(content: &str, author_name: &str) -> bool {
        let content_lower = content.to_lowercase();
        let name_lower = author_name.to_lowercase();

        // 常見垃圾留言特徵
        let spam_keywords = [
            "viagra", "casino", "loan", "mortgage", "porn", "sex",
            "buy now", "click here", "free money", "guaranteed",
            "賺錢", "貸款", "借錢", "免費", "點擊", "色情"
        ];

        // 檢查內容
        for keyword in &spam_keywords {
            if content_lower.contains(keyword) {
                warn!("偵測到疑似垃圾留言關鍵字: {}", keyword);
                return true;
            }
        }

        // 檢查是否全大寫（超過10個字元）
        if content.len() > 10 && content.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) {
            warn!("偵測到疑似垃圾留言: 全大寫");
            return true;
        }

        // 檢查重複字元
        if Self::has_excessive_repetition(content) {
            warn!("偵測到疑似垃圾留言: 過多重複字元");
            return true;
        }

        // 檢查作者名稱
        for keyword in &spam_keywords {
            if name_lower.contains(keyword) {
                warn!("偵測到疑似垃圾留言作者名稱: {}", keyword);
                return true;
            }
        }

        false
    }

    /// 檢查是否有過多重複字元
    fn has_excessive_repetition(content: &str) -> bool {
        let mut current_char = '\0';
        let mut count = 0;

        for c in content.chars() {
            if c == current_char {
                count += 1;
                if count > 5 {
                    return true;
                }
            } else {
                current_char = c;
                count = 1;
            }
        }

        false
    }

    /// 清理留言內容
    fn sanitize_content(content: &str) -> String {
        content
            .trim()
            .replace("<script", "&lt;script")
            .replace("</script>", "&lt;/script&gt;")
            .replace("javascript:", "")
            .chars()
            .filter(|&c| c != '\0' && c != '\x08')
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}