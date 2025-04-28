use crate::error::RepositoryError;
use crate::models::position_category::{
    CreatePositionCategory, PartialPositionCategory, PositionCategory,
};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait PositionCategoryRepository {
    async fn list_position_categories(&self) -> Result<Vec<PositionCategory>>;
    async fn get_position_category_by_id(
        &self,
        position_category_id: i32,
    ) -> Result<PositionCategory, RepositoryError>;
    async fn create_position_category(
        &self,
        new_position_category: CreatePositionCategory,
    ) -> Result<PositionCategory, RepositoryError>;
    async fn delete_position_category(
        &self,
        position_category_id: i32,
    ) -> Result<(), RepositoryError>;
    async fn update_position_category(
        &self,
        position_category_id: i32,
        patch_position_category: PartialPositionCategory,
    ) -> Result<PositionCategory, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgPositionCategoryRepository {
    pub pool_handler: PoolHandler,
}

impl PgPositionCategoryRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }
}

#[async_trait]
impl PositionCategoryRepository for PgPositionCategoryRepository {
    async fn list_position_categories(&self) -> Result<Vec<PositionCategory>> {
        let position_categories = sqlx::query_as!(
            PositionCategory,
            r#"SELECT 
                "id",
                "name"
            FROM "position_category""#
        )
        .fetch_all(self.pool_handler.pool())
        .await?;
        Ok(position_categories)
    }

    async fn get_position_category_by_id(
        &self,
        position_category_id: i32,
    ) -> Result<PositionCategory, RepositoryError> {
        let position_category = sqlx::query_as!(
            PositionCategory,
            r#"SELECT 
                "id",
                "name" 
            FROM "position_category" 
            WHERE "id" = $1"#,
            position_category_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;
        if let Some(position_category) = position_category {
            return Ok(position_category);
        }
        Err(RepositoryError::NotFound)
    }

    async fn create_position_category(
        &self,
        new_position_category: CreatePositionCategory,
    ) -> Result<PositionCategory, RepositoryError> {
        let position_category = sqlx::query_as!(
            PositionCategory,
            r#"INSERT INTO "position_category" (name)
            VALUES ($1) 
            RETURNING "id", "name""#,
            new_position_category.name
        )
        .fetch_one(self.pool_handler.pool())
        .await?;
        Ok(position_category)
    }

    async fn delete_position_category(
        &self,
        position_category_id: i32,
    ) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"DELETE FROM "position_category" WHERE "id" = $1"#,
            position_category_id
        )
        .execute(self.pool_handler.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_position_category(
        &self,
        position_category_id: i32,
        patch_position_category: PartialPositionCategory,
    ) -> Result<PositionCategory, RepositoryError> {
        let position_category = self
            .get_position_category_by_id(position_category_id)
            .await?;

        let name = patch_position_category
            .name
            .unwrap_or(position_category.name);

        let position_category = sqlx::query_as!(
            PositionCategory,
            r#"UPDATE "position_category" 
            SET "name" = $1 
            WHERE "id" = $2 
            RETURNING "id", "name""#,
            name,
            position_category.id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(position_category) = position_category {
            return Ok(position_category);
        }

        Err(RepositoryError::NotFound)
    }
}
