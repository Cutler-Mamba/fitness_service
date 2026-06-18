use fitness_core::error::AppError;
use fitness_entity::tenant;
use sea_orm::*;
use uuid::Uuid;

pub struct TenantService {
    db: DatabaseConnection,
}

impl TenantService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_wechat_id(
        &self,
        wechat_user_id: &str,
    ) -> Result<Option<tenant::Model>, AppError> {
        let t = tenant::Entity::find()
            .filter(tenant::Column::WechatUserId.eq(wechat_user_id))
            .one(&self.db)
            .await?;
        Ok(t)
    }

    pub async fn is_active(&self, wechat_user_id: &str) -> Result<bool, AppError> {
        Ok(self
            .find_by_wechat_id(wechat_user_id)
            .await?
            .is_some_and(|t| t.status == "active"))
    }

    pub async fn create(
        &self,
        wechat_user_id: &str,
        nickname: Option<&str>,
    ) -> Result<tenant::Model, AppError> {
        let existing = self.find_by_wechat_id(wechat_user_id).await?;
        if existing.is_some() {
            return Err(AppError::Conflict("Tenant already exists".into()));
        }

        let now = chrono::Utc::now();
        let new_tenant = tenant::ActiveModel {
            id: Set(Uuid::new_v4()),
            wechat_user_id: Set(wechat_user_id.to_string()),
            nickname: Set(nickname.map(|s| s.to_string())),
            status: Set("active".to_string()),
            daily_quota: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = tenant::Entity::insert(new_tenant)
            .exec_with_returning(&self.db)
            .await?;
        Ok(result)
    }

    pub async fn list_all(&self) -> Result<Vec<tenant::Model>, AppError> {
        let tenants = tenant::Entity::find()
            .order_by_asc(tenant::Column::CreatedAt)
            .all(&self.db)
            .await?;
        Ok(tenants)
    }

    pub async fn set_status(
        &self,
        wechat_user_id: &str,
        status: &str,
    ) -> Result<tenant::Model, AppError> {
        let t = self
            .find_by_wechat_id(wechat_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

        let mut active: tenant::ActiveModel = t.into();
        active.status = Set(status.to_string());
        active.updated_at = Set(chrono::Utc::now());

        let result = active.update(&self.db).await?;
        Ok(result)
    }

    pub async fn find_or_create(
        &self,
        wechat_user_id: &str,
        nickname: Option<&str>,
    ) -> Result<tenant::Model, AppError> {
        if let Some(t) = self.find_by_wechat_id(wechat_user_id).await? {
            return Ok(t);
        }

        let now = chrono::Utc::now();
        let new_tenant = tenant::ActiveModel {
            id: Set(Uuid::new_v4()),
            wechat_user_id: Set(wechat_user_id.to_string()),
            nickname: Set(nickname.map(|s| s.to_string())),
            status: Set("disabled".to_string()),
            daily_quota: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = tenant::Entity::insert(new_tenant)
            .exec_with_returning(&self.db)
            .await?;
        Ok(result)
    }
}
