use crate::error::RepositoryError;
use crate::models::employment::{
    CreateEmployment, Employment, EmploymentState, PartialEmployment, SelectManyFilter,
};
use crate::repositories::event::{EventRepository, PgEventRepository};
use crate::repositories::job_position::{JobPositionRepository, PgJobPositionRepository};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::types::time::OffsetDateTime;
use sqlx::{QueryBuilder, Row};
use std::sync::Arc;

#[async_trait]
pub trait EmploymentRepository {
    async fn list_employment(&self, filter: SelectManyFilter) -> Result<Vec<Employment>>;
    async fn get_employment_by_id(
        &self,
        log_position_id: i32,
    ) -> Result<Employment, RepositoryError>;
    async fn create_employment(
        &self,
        new_log_position: CreateEmployment,
    ) -> Result<Employment, RepositoryError>;
    async fn delete_employment(&self, log_position_id: i32) -> Result<(), RepositoryError>;
    async fn update_employment(
        &self,
        log_position_id: i32,
        patch_log_position: PartialEmployment,
    ) -> Result<Employment, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgEmploymentRepository {
    pub pool_handler: PoolHandler,
}

impl PgEmploymentRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }

    async fn check_add(&self, new_employment: CreateEmployment) -> Result<(), RepositoryError> {
        let job_position_repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(
            self.pool_handler.pool().clone(),
        )));
        let event_repository =
            PgEventRepository::new(PoolHandler::new(Arc::new(self.pool_handler.pool().clone())));

        let job_position = job_position_repository
            .get_job_position_by_id(new_employment.position_id)
            .await?;
        let event = event_repository
            .get_event_by_id(job_position.event_id)
            .await?;

        if event.date_end < OffsetDateTime::now_utc().date() {
            return Err(RepositoryError::GenericError(
                "The event has already ended".to_string(),
            ));
        }

        let existing_record = sqlx::query!(
            r#"SELECT "id" FROM "employment" WHERE "user_id" = $1 AND "position_id" = $2"#,
            new_employment.user_id,
            new_employment.position_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if existing_record.is_some() {
            return Err(RepositoryError::GenericError(
                "The event has already been registered".to_string(),
            ));
        }

        let count_records = sqlx::query!(
            r#"SELECT COUNT(*) FROM "employment" WHERE "position_id" = $1 AND ("state" = 'accepted' OR "state" = 'done')"#,
            new_employment.position_id
        )
            .fetch_one(self.pool_handler.pool())
            .await?;

        let max_val = sqlx::query!(
            r#"SELECT "capacity" FROM "job_position" WHERE "id" = $1"#,
            new_employment.position_id
        )
        .fetch_one(self.pool_handler.pool())
        .await?;

        if (count_records.count.unwrap_or(0) as i32) >= max_val.capacity {
            return Err(RepositoryError::GenericError(
                "Job position is already full".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl EmploymentRepository for PgEmploymentRepository {
    async fn list_employment(&self, filter: SelectManyFilter) -> Result<Vec<Employment>> {
        let mut query_builder = QueryBuilder::new(
            r#"SELECT
                    "id",
                    "rating",
                    "state",
                    "user_id",
                    "position_id"
                FROM "employment"
                WHERE 1=1"#,
        );

        if let Some(rating) = filter.rating {
            query_builder.push(r#" AND "rating" = "#);
            query_builder.push_bind(rating);
        }

        if let Some(user_id) = filter.user_id {
            query_builder.push(r#" AND "user_id" = "#);
            query_builder.push_bind(user_id);
        }

        if let Some(state) = filter.state {
            query_builder.push(r#" AND "state" = "#);
            query_builder.push_bind(state);
        }

        if let Some(position_id) = filter.position_id {
            query_builder.push(r#" AND "position_id" = "#);
            query_builder.push_bind(position_id);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(self.pool_handler.pool()).await?;

        let data: Result<Vec<Employment>, sqlx::Error> = rows
            .into_iter()
            .map(|row| {
                Ok(Employment {
                    id: row.try_get("id")?,
                    rating: row.try_get("rating")?,
                    state: row.try_get("state")?,
                    user_id: row.try_get("user_id")?,
                    position_id: row.try_get("position_id")?,
                })
            })
            .collect();

        let data = data?;
        Ok(data)
    }

    async fn get_employment_by_id(
        &self,
        employment_id: i32,
    ) -> Result<Employment, RepositoryError> {
        let employment = sqlx::query_as!(
            Employment,
            r#"SELECT
                "id",
                "rating",
                "state" AS "state: EmploymentState",
                "user_id",
                "position_id"
            FROM "employment" WHERE "id" = $1"#,
            employment_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;
        if let Some(employment) = employment {
            return Ok(employment);
        }
        Err(RepositoryError::NotFound)
    }

    async fn create_employment(
        &self,
        new_employment: CreateEmployment,
    ) -> Result<Employment, RepositoryError> {
        Self::check_add(self, new_employment.clone()).await?;
        let employment = sqlx::query_as!(
            Employment,
            r#"INSERT INTO "employment" ("rating", "state", "user_id", "position_id")
            VALUES ($1, $2::employment_state, $3, $4)
            RETURNING
                "id", "rating", "state" as "state: EmploymentState", "user_id", "position_id""#,
            new_employment.rating,
            new_employment.state as _,
            new_employment.user_id,
            new_employment.position_id
        )
        .fetch_one(self.pool_handler.pool())
        .await?;
        Ok(employment)
    }

    async fn delete_employment(&self, employment_id: i32) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM "employment" WHERE "id" = $1"#, employment_id)
            .execute(self.pool_handler.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_employment(
        &self,
        employment_id: i32,
        patch_log_position: PartialEmployment,
    ) -> Result<Employment, RepositoryError> {
        let employment = self.get_employment_by_id(employment_id).await?;

        let rating = patch_log_position.rating.unwrap_or(employment.rating);
        let state = patch_log_position.state.unwrap_or(employment.state);
        let user_id = patch_log_position.user_id.unwrap_or(employment.user_id);
        let position_id = patch_log_position
            .position_id
            .unwrap_or(employment.position_id);

        let employment = sqlx::query_as!(
            Employment,
            r#"UPDATE "employment" SET
                "rating" = $2, 
                "state" = $3::employment_state, 
                "user_id" = $4, 
                "position_id" = $5 
            WHERE "id" = $1
            RETURNING
                "id", "rating", "state" as "state: EmploymentState", "user_id", "position_id"
            "#,
            employment_id,
            rating,
            state as _,
            user_id,
            position_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(employment) = employment {
            return Ok(employment);
        }
        Err(RepositoryError::NotFound)
    }
}
