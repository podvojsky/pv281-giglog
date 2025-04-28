use crate::error::RepositoryError;
use crate::models::employment::EmploymentState;
use crate::models::worked_hours::{
    CreateWorkedHours, PartialWorkedHours, SelectManyFilter, WorkedHours,
};
use crate::repositories::employment::{EmploymentRepository, PgEmploymentRepository};
use crate::repositories::event::{EventRepository, PgEventRepository};
use crate::repositories::job_position::{JobPositionRepository, PgJobPositionRepository};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};
use std::sync::Arc;

#[async_trait]
#[allow(dead_code)]
pub trait WorkedHoursRepository {
    async fn list_worked_hours(
        &self,
        filter: SelectManyFilter,
    ) -> Result<Vec<WorkedHours>, RepositoryError>;
    async fn get_worked_hours_by_id(&self, worked_id: i32) -> Result<WorkedHours, RepositoryError>;
    async fn create_worked_hours(
        &self,
        new_worked: CreateWorkedHours,
    ) -> Result<WorkedHours, RepositoryError>;
    async fn delete_worked_hours(&self, worked_id: i32) -> Result<(), RepositoryError>;
    async fn update_worked_hours(
        &self,
        worked_id: i32,
        patch_worked: PartialWorkedHours,
    ) -> Result<WorkedHours, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgWorkedHoursRepository {
    pub pool_handler: PoolHandler,
}

impl PgWorkedHoursRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }

    async fn check_add(&self, new_worked: CreateWorkedHours) -> Result<(), RepositoryError> {
        let employment_repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(
            self.pool_handler.pool().clone(),
        )));
        let job_position_repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(
            self.pool_handler.pool().clone(),
        )));
        let event_repository =
            PgEventRepository::new(PoolHandler::new(Arc::new(self.pool_handler.pool().clone())));

        let employment = employment_repository
            .get_employment_by_id(new_worked.employment_id)
            .await?;
        let job_position = job_position_repository
            .get_job_position_by_id(employment.position_id)
            .await?;
        let event = event_repository
            .get_event_by_id(job_position.event_id)
            .await?;

        if employment.state != EmploymentState::Accepted {
            return Err(RepositoryError::GenericError(
                "logging hours job in wrong state".to_string(),
            ));
        }

        if event.date_start > new_worked.date || event.date_end < new_worked.date {
            return Err(RepositoryError::GenericError(
                "logging hours date outside of events date".to_string(),
            ));
        }

        let result = sqlx::query!(
            r#"SELECT * FROM "worked_hours" WHERE "employment_id" = $1 AND "date" = $2"#,
            new_worked.employment_id,
            new_worked.date
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if result.is_some() {
            return Err(RepositoryError::GenericError(
                "cannot log worked hours for one job in same day twice".to_string(),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl WorkedHoursRepository for PgWorkedHoursRepository {
    async fn list_worked_hours(
        &self,
        filter: SelectManyFilter,
    ) -> Result<Vec<WorkedHours>, RepositoryError> {
        let mut query_builder = QueryBuilder::new(
            r#"SELECT 
                    "id",
                    "date",
                    "hours_worked",
                    "employment_id"                                        
                FROM "worked_hours" WHERE 1=1"#,
        );

        if let Some(registration_id) = filter.employment_id {
            query_builder.push(r#" AND "employment_id" = "#);
            query_builder.push_bind(registration_id);
        }

        if let Some(hours_worked) = filter.hours_worked {
            query_builder.push(r#" AND "hours_worked" = "#);
            query_builder.push_bind(hours_worked);
        }

        if let Some(date) = filter.date {
            query_builder.push(r#" AND "date" = "#);
            query_builder.push_bind(date);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(self.pool_handler.pool()).await?;

        let data: Result<Vec<WorkedHours>, sqlx::Error> = rows
            .into_iter()
            .map(|row| {
                Ok(WorkedHours {
                    id: row.try_get("id")?,
                    date: row.try_get("date")?,
                    hours_worked: row.try_get("hours_worked")?,
                    employment_id: row.try_get("employment_id")?,
                })
            })
            .collect();

        let data = data?;
        Ok(data)
    }

    async fn get_worked_hours_by_id(&self, worked_id: i32) -> Result<WorkedHours, RepositoryError> {
        let worked = sqlx::query_as!(
            WorkedHours,
            r#"
            SELECT
                "id",
                "date",
                "hours_worked",
                "employment_id"
            FROM "worked_hours"
            WHERE "id" = $1
            "#,
            worked_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(w) = worked {
            return Ok(w);
        }
        Err(RepositoryError::NotFound)
    }

    async fn create_worked_hours(
        &self,
        new_worked: CreateWorkedHours,
    ) -> Result<WorkedHours, RepositoryError> {
        Self::check_add(self, new_worked.clone()).await?;
        let worked = sqlx::query_as!(
            WorkedHours,
            r#"INSERT INTO "worked_hours" ("date", "hours_worked", "employment_id")
            VALUES ($1, $2, $3) 
            RETURNING 
                "id",
                "date",
                "hours_worked",
                "employment_id""#,
            new_worked.date,
            new_worked.hours_worked,
            new_worked.employment_id
        )
        .fetch_one(self.pool_handler.pool())
        .await?;
        Ok(worked)
    }

    async fn delete_worked_hours(&self, worked_id: i32) -> Result<(), RepositoryError> {
        let result = sqlx::query_as!(
            WorkedHours,
            r#"DELETE FROM "worked_hours" WHERE "id" = $1;"#,
            worked_id
        )
        .execute(self.pool_handler.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_worked_hours(
        &self,
        worked_id: i32,
        patch_worked: PartialWorkedHours,
    ) -> Result<WorkedHours, RepositoryError> {
        let worked_hours = self.get_worked_hours_by_id(worked_id).await?;

        let hours_worked = patch_worked
            .hours_worked
            .unwrap_or(worked_hours.hours_worked);
        let date = patch_worked.date.unwrap_or(worked_hours.date);
        let employment_id = patch_worked
            .employment_id
            .unwrap_or(worked_hours.employment_id);

        let worked_hours = sqlx::query_as!(
            WorkedHours,
            r#"UPDATE "worked_hours"
            SET 
                "employment_id" = $2, 
                "hours_worked" = $3, 
                "date" = $4 
            WHERE "id" = $1 
            RETURNING
                "id", 
                "employment_id",
                "hours_worked",
                "date"
            "#,
            worked_id,
            employment_id,
            hours_worked,
            date
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(w) = worked_hours {
            return Ok(w);
        }
        Err(RepositoryError::NotFound)
    }
}
