use crate::error::RepositoryError;
use crate::models::event_manager_relation::{CreateEventManagerRelation, EventManagerRelation};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EventManagerRelationRepository {
    async fn create_relation(
        &self,
        new_relation: CreateEventManagerRelation,
    ) -> Result<EventManagerRelation, RepositoryError>;
    async fn delete_relation(&self, relation: EventManagerRelation) -> Result<(), RepositoryError>;
    async fn list_event_managers(
        &self,
        event_id: i32,
    ) -> Result<Vec<EventManagerRelation>, RepositoryError>;
    async fn list_managers_events(
        &self,
        user_id: i32,
    ) -> Result<Vec<EventManagerRelation>, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgEventManagerRelationRepository {
    pub pool_handler: PoolHandler,
}

impl PgEventManagerRelationRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }

    async fn check_add(
        &self,
        new_relation: CreateEventManagerRelation,
    ) -> Result<(), RepositoryError> {
        let existing_record = sqlx::query!(
            r#"SELECT * FROM "event_manager_relation" WHERE "event_id"=$1 AND "user_id"=$2"#,
            new_relation.event_id,
            new_relation.user_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if existing_record.is_some() {
            return Err(RepositoryError::GenericError(
                "The event has already been registered".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl EventManagerRelationRepository for PgEventManagerRelationRepository {
    async fn create_relation(
        &self,
        new_relation: CreateEventManagerRelation,
    ) -> Result<EventManagerRelation, RepositoryError> {
        Self::check_add(self, new_relation.clone()).await?;
        let relation = sqlx::query_as!(
            EventManagerRelation,
            r#"INSERT INTO "event_manager_relation" ("user_id", "event_id")
            VALUES ($1, $2)
            RETURNING 
                "user_id",
                "event_id"
            "#,
            new_relation.user_id,
            new_relation.event_id
        )
        .fetch_one(self.pool_handler.pool())
        .await?;

        Ok(relation)
    }

    async fn delete_relation(&self, relation: EventManagerRelation) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"DELETE FROM "event_manager_relation" 
            WHERE "user_id" = $1 AND "event_id" = $2
            "#,
            relation.user_id,
            relation.event_id
        )
        .execute(self.pool_handler.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn list_event_managers(
        &self,
        event_id: i32,
    ) -> Result<Vec<EventManagerRelation>, RepositoryError> {
        let relations = sqlx::query_as!(
            EventManagerRelation,
            r#"SELECT
                "user_id", "event_id"
            FROM "event_manager_relation"
            WHERE "event_id" = $1"#,
            event_id
        )
        .fetch_all(self.pool_handler.pool())
        .await?;

        Ok(relations)
    }

    async fn list_managers_events(
        &self,
        user_id: i32,
    ) -> Result<Vec<EventManagerRelation>, RepositoryError> {
        let relations = sqlx::query_as!(
            EventManagerRelation,
            r#"SELECT
                "user_id", "event_id"
            FROM "event_manager_relation"
            WHERE "user_id" = $1"#,
            user_id
        )
        .fetch_all(self.pool_handler.pool())
        .await?;

        Ok(relations)
    }
}
