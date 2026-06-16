use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use fitness_core::error::AppError;
use fitness_entity::user;
use sea_orm::*;
use uuid::Uuid;

pub struct UserService {
    db: DatabaseConnection,
}

impl UserService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn register(
        &self,
        email: Option<String>,
        phone: Option<String>,
        password: String,
        nickname: String,
    ) -> Result<user::Model, AppError> {
        if email.is_none() && phone.is_none() {
            return Err(AppError::BadRequest("Email or phone is required".into()));
        }

        if let Some(ref e) = email {
            let existing = user::Entity::find()
                .filter(user::Column::Email.eq(e))
                .one(&self.db)
                .await?;
            if existing.is_some() {
                return Err(AppError::Conflict("Email already registered".into()));
            }
        }

        if let Some(ref p) = phone {
            let existing = user::Entity::find()
                .filter(user::Column::Phone.eq(p))
                .one(&self.db)
                .await?;
            if existing.is_some() {
                return Err(AppError::Conflict("Phone already registered".into()));
            }
        }

        let password_hash = hash_password(&password)?;

        let now = chrono::Utc::now();
        let new_user = user::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(email),
            phone: Set(phone),
            password_hash: Set(Some(password_hash)),
            nickname: Set(nickname),
            avatar: Set(None),
            fitness_level: Set(None),
            gender: Set(None),
            birth_date: Set(None),
            height: Set(None),
            weight: Set(None),
            feishu_open_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = user::Entity::insert(new_user)
            .exec_with_returning(&self.db)
            .await?;

        Ok(result)
    }

    pub async fn login(&self, account: &str, password: &str) -> Result<user::Model, AppError> {
        let user = user::Entity::find()
            .filter(
                Condition::any()
                    .add(user::Column::Email.eq(account))
                    .add(user::Column::Phone.eq(account)),
            )
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

        let password_hash = user
            .password_hash
            .as_ref()
            .ok_or_else(|| AppError::Unauthorized("No password set for this account".into()))?;

        verify_password(password, password_hash)?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<user::Model, AppError> {
        user::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".into()))
    }

    pub async fn find_by_feishu_open_id(
        &self,
        open_id: &str,
    ) -> Result<Option<user::Model>, AppError> {
        let user = user::Entity::find()
            .filter(user::Column::FeishuOpenId.eq(open_id))
            .one(&self.db)
            .await?;
        Ok(user)
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        nickname: Option<String>,
        avatar: Option<String>,
        fitness_level: Option<String>,
        gender: Option<String>,
        birth_date: Option<chrono::NaiveDate>,
        height: Option<rust_decimal::Decimal>,
        weight: Option<rust_decimal::Decimal>,
    ) -> Result<user::Model, AppError> {
        let mut user_active: user::ActiveModel = self.find_by_id(user_id).await?.into();

        if let Some(n) = nickname {
            user_active.nickname = Set(n);
        }
        if let Some(a) = avatar {
            user_active.avatar = Set(Some(a));
        }
        if let Some(l) = fitness_level {
            user_active.fitness_level = Set(Some(l));
        }
        if let Some(g) = gender {
            user_active.gender = Set(Some(g));
        }
        if let Some(b) = birth_date {
            user_active.birth_date = Set(Some(b));
        }
        if let Some(h) = height {
            user_active.height = Set(Some(h));
        }
        if let Some(w) = weight {
            user_active.weight = Set(Some(w));
        }
        user_active.updated_at = Set(chrono::Utc::now());

        let result = user_active.update(&self.db).await?;
        Ok(result)
    }

    pub async fn bind_feishu(&self, user_id: Uuid, open_id: &str) -> Result<(), AppError> {
        let mut user_active: user::ActiveModel = self.find_by_id(user_id).await?.into();
        user_active.feishu_open_id = Set(Some(open_id.to_string()));
        user_active.updated_at = Set(chrono::Utc::now());
        user_active.update(&self.db).await?;
        Ok(())
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;
    Ok(())
}
