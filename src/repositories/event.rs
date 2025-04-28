use crate::error::RepositoryError;
use crate::models::event::{CreateEvent, Event, PartialEvent, SelectManyFilter};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};

#[async_trait]
pub trait EventRepository {
    async fn list_events(&self, filter: SelectManyFilter) -> Result<Vec<Event>>;
    async fn get_event_by_id(&self, event_id: i32) -> Result<Event, RepositoryError>;
    async fn create_event(&self, new_event: CreateEvent) -> Result<Event, RepositoryError>;
    async fn delete_event(&self, event_id: i32) -> Result<(), RepositoryError>;
    async fn update_event(
        &self,
        event_id: i32,
        patch_event: PartialEvent,
    ) -> Result<Event, RepositoryError>;
    async fn list_events_worked_by_user(&self, user_id: i32)
        -> Result<Vec<Event>, RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PgEventRepository {
    pub pool_handler: PoolHandler,
}

impl PgEventRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }

    fn filters_by_state(state: Option<String>, city: Option<String>) -> bool {
        if state.is_none() && city.is_none() {
            return false;
        }
        true
    }
}

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn list_events(&self, filter: SelectManyFilter) -> Result<Vec<Event>> {
        let mut query_builder = QueryBuilder::new(
            r#"SELECT
                "event"."id",
                "event"."name",
                "event"."date_start",
                "event"."date_end",
                "event"."img_url",
                "event"."description",
                "event"."is_draft",
                "event"."venue_id",
                "event"."owner_id"
            FROM "event""#,
        );

        if Self::filters_by_state(filter.state.clone(), filter.city.clone()) {
            query_builder.push(r#" JOIN "venue" ON "event"."venue_id" = "venue"."id""#);
        }

        query_builder.push(r#" WHERE 1=1"#);

        if let Some(venue_id) = filter.venue_id {
            query_builder.push(r#" AND "event"."venue_id" = "#);
            query_builder.push_bind(venue_id);
        }

        if let Some(owner_id) = filter.owner_id {
            query_builder.push(r#" AND "event"."owner_id" = "#);
            query_builder.push_bind(owner_id);
        }

        if let Some(is_draft) = filter.is_draft {
            query_builder.push(r#" AND "event"."is_draft" = "#);
            query_builder.push_bind(is_draft);
        }

        if let Some(date_from) = filter.date_from {
            query_builder.push(r#" AND "event"."date_start" >= "#);
            query_builder.push_bind(date_from);
        }

        if let Some(date_to) = filter.date_to {
            query_builder.push(r#" AND "event"."date_end" <= "#);
            query_builder.push_bind(date_to);
        }

        if let Some(state) = filter.state {
            query_builder.push(r#" AND "venue"."state" = "#);
            query_builder.push_bind(state);
        }

        if let Some(city) = filter.city {
            query_builder.push(r#" AND "venue"."town" = "#);
            query_builder.push_bind(city);
        }

        if let Some(name) = filter.name {
            query_builder.push(r#" AND "event"."name" ILIKE '%' || "#);
            query_builder.push_bind(name);
            query_builder.push(r#" || '%'"#);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(self.pool_handler.pool()).await?;

        let data: Result<Vec<Event>, sqlx::Error> = rows
            .into_iter()
            .map(|row| {
                Ok(Event {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    date_start: row.try_get("date_start")?,
                    date_end: row.try_get("date_end")?,
                    img_url: row.try_get("img_url")?,
                    description: row.try_get("description")?,
                    is_draft: row.try_get("is_draft")?,
                    venue_id: row.try_get("venue_id")?,
                    owner_id: row.try_get("owner_id")?,
                })
            })
            .collect();

        let data = data?;
        Ok(data)
    }

    async fn get_event_by_id(&self, event_id: i32) -> Result<Event, RepositoryError> {
        let event = sqlx::query_as!(
            Event,
            r#"SELECT
                "id",
                "name", 
                "date_start", 
                "date_end", "img_url", 
                "description", 
                "is_draft", 
                "venue_id", 
                "owner_id" 
            FROM "event" WHERE "id" = $1"#,
            event_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(event) = event {
            return Ok(event);
        }

        Err(RepositoryError::NotFound)
    }

    async fn create_event(&self, new_event: CreateEvent) -> Result<Event, RepositoryError> {
        let event = sqlx::query_as!(
            Event,
            r#"INSERT INTO "event"
            ("name", "date_start", "date_end", "img_url", "description", "is_draft", "venue_id", "owner_id")
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING
            "id","name", "date_start", "date_end", "img_url", "description", "is_draft", "venue_id", "owner_id""#,
            new_event.name,
            new_event.date_start,
            new_event.date_end,
            new_event.img_url,
            new_event.description,
            new_event.is_draft,
            new_event.venue_id,
            new_event.owner_id,
        )
            .fetch_one(self.pool_handler.pool())
            .await?;
        Ok(event)
    }

    async fn delete_event(&self, event_id: i32) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM "event" WHERE "id" = $1"#, event_id)
            .execute(self.pool_handler.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_event(
        &self,
        event_id: i32,
        patch_event: PartialEvent,
    ) -> Result<Event, RepositoryError> {
        let event = self.get_event_by_id(event_id).await?;

        let name = patch_event.name.unwrap_or(event.name);
        let date_start = patch_event.date_start.unwrap_or(event.date_start);
        let date_end = patch_event.date_end.unwrap_or(event.date_end);
        let img_url = patch_event.img_url.unwrap_or(event.img_url);
        let description = patch_event.description.or(event.description);
        let is_draft = patch_event.is_draft.unwrap_or(false);
        let venue_id = patch_event.venue_id.unwrap_or(event.venue_id);
        let owner_id = patch_event.owner_id.unwrap_or(event.owner_id);

        let event = sqlx::query_as!(
            Event,
            r#"UPDATE "event" SET
                "name" = $2, 
                "date_start" = $3, 
                "date_end" = $4, 
                "img_url" = $5, 
                "description" = $6, 
                "is_draft" = $7, 
                "venue_id" = $8, 
                "owner_id" = $9 
            WHERE "id" = $1                
            RETURNING
                "id", 
                "name",
                "date_start",
                "date_end",
                "img_url",
                "description",
                "is_draft",
                "venue_id",
                "owner_id"
            "#,
            event_id,
            name,
            date_start,
            date_end,
            img_url,
            description,
            is_draft,
            venue_id,
            owner_id,
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(event) = event {
            return Ok(event);
        }
        Err(RepositoryError::NotFound)
    }

    async fn list_events_worked_by_user(
        &self,
        user_id: i32,
    ) -> Result<Vec<Event>, RepositoryError> {
        let events = sqlx::query_as!(
            Event,
            r#"
            SELECT DISTINCT "event".*
            FROM "event"
            JOIN "job_position" ON "job_position"."event_id"="event"."id"
            JOIN "employment" ON "employment"."position_id"="job_position"."id"
            WHERE "employment"."state"='accepted' AND "employment"."user_id"=$1
            ;"#,
            user_id
        )
        .fetch_all(self.pool_handler.pool())
        .await?;

        Ok(events)
    }
}
