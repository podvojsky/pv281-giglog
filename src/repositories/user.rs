use crate::error::RepositoryError;
use crate::models::user::{CreateUser, Gender, PartialUser, SelectManyFilter, User, UserRole};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};

#[async_trait]
pub trait UserRepository {
    async fn list_users(&self, filter: SelectManyFilter) -> Result<Vec<User>>;
    async fn get_user_by_id(&self, user_id: i32) -> Result<User, RepositoryError>;
    async fn get_user_by_username(&self, username: String) -> Result<User, RepositoryError>;
    async fn create_user(&self, new_user: CreateUser) -> Result<User, RepositoryError>;
    async fn delete_user(&self, user_id: i32) -> Result<(), RepositoryError>;
    async fn update_user(
        &self,
        user_id: i32,
        patch_user: PartialUser,
    ) -> Result<User, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgUserRepository {
    pub pool_handler: PoolHandler,
}

impl PgUserRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn list_users(&self, filter: SelectManyFilter) -> Result<Vec<User>> {
        let mut query_builder = QueryBuilder::new(r#"SELECT * FROM "user" WHERE 1=1"#);

        if let Some(username) = filter.username {
            query_builder.push(" AND username = ");
            query_builder.push_bind(username);
        }

        if let Some(role) = filter.role {
            query_builder.push(" AND role = ");
            query_builder.push_bind(role);
        }

        if let Some(gender) = filter.gender {
            query_builder.push(" AND gender = ");
            query_builder.push_bind(gender);
        }

        if let Some(first_name) = filter.first_name {
            query_builder.push(" AND first_name = ");
            query_builder.push_bind(first_name);
        }

        if let Some(last_name) = filter.last_name {
            query_builder.push(" AND last_name = ");
            query_builder.push_bind(last_name);
        }

        if let Some(tax_rate) = filter.tax_rate {
            query_builder.push(" AND tax_rate = ");
            query_builder.push_bind(tax_rate);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(self.pool_handler.pool()).await?;

        let data: Result<Vec<User>, sqlx::Error> = rows
            .into_iter()
            .map(|row| {
                Ok(User {
                    id: row.try_get("id")?,
                    first_name: row.try_get("first_name")?,
                    last_name: row.try_get("last_name")?,
                    username: row.try_get("username")?,
                    gender: row.try_get("gender")?,
                    birth_date: row.try_get("birth_date")?,
                    email: row.try_get("email")?,
                    phone: row.try_get("phone")?,
                    password_hash: row.try_get("password_hash")?,
                    role: row.try_get("role")?,
                    tax_rate: row.try_get("tax_rate")?,
                    avatar_url: row.try_get("avatar_url")?,
                })
            })
            .collect();
        let data = data?;
        Ok(data)
    }

    async fn get_user_by_id(&self, user_id: i32) -> Result<User, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                "id",
                "first_name",
                "last_name",
                "username",
                "gender" AS "gender: Gender",
                "role" AS "role: UserRole",
                "birth_date",
                "tax_rate",
                "email",
                "phone",
                "password_hash",
                "avatar_url"
            FROM "user"
            WHERE "id" = $1;
            "#,
            user_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(user) = user {
            return Ok(user);
        }

        Err(RepositoryError::NotFound)
    }

    async fn get_user_by_username(&self, username: String) -> Result<User, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                "id",
                "first_name",
                "last_name",
                "username",
                "gender" AS "gender: Gender",
                "role" AS "role: UserRole",
                "birth_date",
                "tax_rate",
                "email",
                "phone",
                "password_hash",
                "avatar_url"
            FROM "user"
            WHERE "username" = $1"#,
            username
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(user) = user {
            return Ok(user);
        }

        Err(RepositoryError::NotFound)
    }

    async fn create_user(&self, new_user: CreateUser) -> Result<User, RepositoryError> {
        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO "user" (
                "first_name", "last_name", "username", "gender", "birth_date", "email", "phone", "password_hash", "role", "tax_rate", "avatar_url"
            )
            VALUES ($1, $2, $3, $4::gender_type, $5, $6, $7, $8, $9::user_role, $10, $11)
            RETURNING
                "id", "first_name", "last_name", "username", "gender" as "gender: Gender", "birth_date", "email", "phone", "password_hash", "role" as "role: UserRole", "tax_rate", "avatar_url"
            "#,
            new_user.first_name,
            new_user.last_name,
            new_user.username,
            new_user.gender as _,
            new_user.birth_date,
            new_user.email,
            new_user.phone,
            new_user.password_hash,
            new_user.role as _,
            new_user.tax_rate,
            new_user.avatar_url
        )
            .fetch_one(self.pool_handler.pool())
            .await?;
        Ok(user)
    }

    async fn delete_user(&self, user_id: i32) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM "user" WHERE "id" = $1"#, user_id)
            .execute(self.pool_handler.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_user(
        &self,
        user_id: i32,
        patch_user: PartialUser,
    ) -> Result<User, RepositoryError> {
        let user = self.get_user_by_id(user_id).await?;

        let first_name = patch_user.first_name.unwrap_or(user.first_name);
        let last_name = patch_user.last_name.unwrap_or(user.last_name);
        let username = patch_user.username.unwrap_or(user.username);
        let gender = patch_user.gender.unwrap_or(user.gender);
        let birth_date = patch_user.birth_date.unwrap_or(user.birth_date);
        let email = patch_user.email.unwrap_or(user.email);
        let phone = patch_user.phone.unwrap_or(user.phone);
        let password_hash = patch_user.password_hash.unwrap_or(user.password_hash);
        let role = patch_user.role.unwrap_or(user.role);
        let tax_rate = patch_user.tax_rate.unwrap_or(user.tax_rate);
        let avatar_url = patch_user.avatar_url.or(user.avatar_url);

        let user = sqlx::query_as!(
            User,
            r#"UPDATE "user" SET
                "first_name" = $2, 
                "last_name" = $3, 
                "username" = $4, 
                "gender" = $5::gender_type, 
                "birth_date" = $6, 
                "email" = $7, 
                "phone" = $8, 
                "password_hash" = $9, 
                "role" = $10::user_role, 
                "tax_rate" = $11 ,
                "avatar_url" = $12
            WHERE "id" = $1
            RETURNING
                "id", 
                "first_name", 
                "last_name", 
                "username", 
                "gender" as "gender: Gender", 
                "birth_date", 
                "email", 
                "phone", 
                "password_hash", 
                "role" as "role: UserRole", 
                "tax_rate",
                "avatar_url"
            "#,
            user_id,
            first_name,
            last_name,
            username,
            gender as _,
            birth_date,
            email,
            phone,
            password_hash,
            role as _,
            tax_rate,
            avatar_url
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(user) = user {
            return Ok(user);
        }

        Err(RepositoryError::NotFound)
    }
}
