use crate::error::RepositoryError;
use crate::models::job_position::{
    CreateJobPosition, JobPosition, PartialJobPosition, SalaryCurrency, SelectManyFilter,
};
use crate::repositories::event::{EventRepository, PgEventRepository};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use sqlx::types::time::OffsetDateTime;
use sqlx::{QueryBuilder, Row};
use std::sync::Arc;
use async_trait::async_trait;

#[async_trait]
pub trait JobPositionRepository {
    async fn list_job_positions(&self, filter: SelectManyFilter) -> Result<Vec<JobPosition>>;
    async fn list_job_positions_worked_by_user_on_event(
        &self,
        user_id: i32,
        event_id: i32,
    ) -> Result<Vec<JobPosition>>;
    async fn get_job_position_by_id(
        &self,
        position_id: i32,
    ) -> Result<JobPosition, RepositoryError>;
    async fn create_job_position(
        &self,
        new_position: CreateJobPosition,
    ) -> Result<JobPosition, RepositoryError>;
    async fn delete_job_position(&self, position_id: i32) -> Result<(), RepositoryError>;
    async fn update_job_position(
        &self,
        position_id: i32,
        patch_position: PartialJobPosition,
    ) -> Result<JobPosition, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgJobPositionRepository {
    pub pool_handler: PoolHandler,
}

impl PgJobPositionRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }

    async fn check_add(&self, new_position: CreateJobPosition) -> Result<(), RepositoryError> {
        let event_repository =
            PgEventRepository::new(PoolHandler::new(Arc::new(self.pool_handler.pool().clone())));

        let event = event_repository
            .get_event_by_id(new_position.event_id)
            .await?;

        if event.date_end < OffsetDateTime::now_utc().date() {
            return Err(RepositoryError::GenericError(
                "The event has already ended".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl JobPositionRepository for PgJobPositionRepository {
    async fn list_job_positions(&self, filter: SelectManyFilter) -> Result<Vec<JobPosition>> {
        let mut query_builder = QueryBuilder::new(
            r#"SELECT
                    "id",
                    "name",
                    "description",
                    "salary",
                    "capacity",
                    "instructions_html",
                    "is_opened_for_registration",
                    "currency",
                    "event_id",
                    "position_category_id"
                FROM "job_position" WHERE 1=1"#,
        );

        if let Some(capacity) = filter.capacity {
            query_builder.push(r#" AND "capacity" = "#);
            query_builder.push_bind(capacity);
        }

        if let Some(position_category_id) = filter.position_category_id {
            query_builder.push(r#" AND "position_category_id" = "#);
            query_builder.push_bind(position_category_id);
        }

        if let Some(event_id) = filter.event_id {
            query_builder.push(r#" AND "event_id" = "#);
            query_builder.push_bind(event_id);
        }

        if let Some(salary) = filter.salary {
            query_builder.push(r#" AND "salary" = "#);
            query_builder.push_bind(salary);
        }

        if let Some(is_opened_for_registration) = filter.is_opened_for_registration {
            query_builder.push(r#" AND "is_opened_for_registration" = "#);
            query_builder.push_bind(is_opened_for_registration);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(self.pool_handler.pool()).await?;

        let data: Result<Vec<JobPosition>, sqlx::Error> = rows
            .into_iter()
            .map(|row| {
                Ok(JobPosition {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    salary: row.try_get("salary")?,
                    currency: row.try_get("currency")?,
                    capacity: row.try_get("capacity")?,
                    instructions_html: row.try_get("instructions_html")?,
                    is_opened_for_registration: row.try_get("is_opened_for_registration")?,
                    event_id: row.try_get("event_id")?,
                    position_category_id: row.try_get("position_category_id")?,
                })
            })
            .collect();

        let data = data?;
        Ok(data)
    }

    async fn get_job_position_by_id(
        &self,
        position_id: i32,
    ) -> Result<JobPosition, RepositoryError> {
        let job_position = sqlx::query_as!(
            JobPosition,
            r#"SELECT
                "id",
                "name",
                "description",
                "salary",
                "currency" AS "currency: SalaryCurrency",
                "capacity",
                "instructions_html",
                "is_opened_for_registration",
                "event_id",
                "position_category_id"
            FROM "job_position"
            WHERE "id" = $1"#,
            position_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(job_position) = job_position {
            return Ok(job_position);
        }

        Err(RepositoryError::NotFound)
    }

    async fn create_job_position(
        &self,
        new_position: CreateJobPosition,
    ) -> Result<JobPosition, RepositoryError> {
        Self::check_add(self, new_position.clone()).await?;
        let job_position = sqlx::query_as!(
            JobPosition,
            r#"INSERT INTO "job_position"
            ("name", "description", "salary", "capacity", "instructions_html", "is_opened_for_registration", "currency", "event_id", "position_category_id")
            VALUES ($1, $2, $3, $4, $5, $6, $7::salary_currency, $8, $9)
            RETURNING "id", "name", "description", "salary", "capacity", "instructions_html", "is_opened_for_registration", "currency" as "currency: SalaryCurrency", "event_id", "position_category_id";"#,
            new_position.name,
            new_position.description,
            new_position.salary,
            new_position.capacity,
            new_position.instructions_html,
            new_position.is_opened_for_registration,
            new_position.currency as _,
            new_position.event_id,
            new_position.position_category_id
        )
            .fetch_one(self.pool_handler.pool())
            .await?;
        Ok(job_position)
    }

    async fn delete_job_position(&self, position_id: i32) -> Result<(), RepositoryError> {
        let result = sqlx::query_as!(
            JobPosition,
            r#"DELETE FROM "job_position" WHERE "id" = $1"#,
            position_id
        )
        .execute(self.pool_handler.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_job_position(
        &self,
        position_id: i32,
        patch_position: PartialJobPosition,
    ) -> Result<JobPosition, RepositoryError> {
        let job_position = self.get_job_position_by_id(position_id).await?;

        let name = patch_position.name.unwrap_or(job_position.name);
        let description = patch_position.description.or(job_position.description);
        let salary = patch_position.salary.unwrap_or(job_position.salary);
        let capacity = patch_position.capacity.unwrap_or(job_position.capacity);
        let currency = patch_position.currency.unwrap_or(job_position.currency);
        let instructions_html = patch_position
            .instructions_html
            .unwrap_or(job_position.instructions_html);
        let is_opened_for_registration = patch_position
            .is_opened_for_registration
            .or(Some(job_position.is_opened_for_registration));
        let event_id = patch_position.event_id.unwrap_or(job_position.event_id);
        let position_category_id = patch_position
            .position_category_id
            .unwrap_or(job_position.position_category_id);

        let job_position = sqlx::query_as!(
            JobPosition,
            r#"UPDATE "job_position" SET
                "name" = $2,
                "description" = $3,
                "salary" = $4,
                "capacity" = $5,
                "currency" = $6::salary_currency,
                "instructions_html" = $7,
                "is_opened_for_registration" = $8,
                "event_id" = $9,
                "position_category_id" = $10
            WHERE id = $1
            RETURNING 
                "id", 
                "name", 
                "description", 
                "salary", 
                "capacity", 
                "instructions_html", 
                "is_opened_for_registration", 
                "currency" as "currency: SalaryCurrency", 
                "event_id", 
                "position_category_id""#,
            position_id,
            name,
            description,
            salary,
            capacity,
            currency as _,
            instructions_html,
            is_opened_for_registration,
            event_id,
            position_category_id,
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(job_position) = job_position {
            return Ok(job_position);
        }
        Err(RepositoryError::NotFound)
    }

    async fn list_job_positions_worked_by_user_on_event(
        &self,
        user_id: i32,
        event_id: i32,
    ) -> Result<Vec<JobPosition>> {
        let jobs = sqlx::query_as!(
            JobPosition,
            r#"
            SELECT DISTINCT
                "job_position"."id",
                "job_position"."name",
                "job_position"."description",
                "job_position"."salary",
                "job_position"."currency" AS "currency: SalaryCurrency",
                "job_position"."capacity",
                "job_position"."instructions_html",
                "job_position"."is_opened_for_registration",
                "job_position"."event_id",
                "job_position"."position_category_id"
            FROM "job_position"
            JOIN "employment" ON "employment"."position_id"="job_position"."id"
            JOIN "event" ON "job_position"."event_id"="event"."id"
            WHERE "employment"."user_id"=$1 AND "event"."id"=$2
            ;"#,
            user_id,
            event_id
        )
        .fetch_all(self.pool_handler.pool())
        .await?;

        Ok(jobs)
    }
}
