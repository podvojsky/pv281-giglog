use crate::error::RepositoryError;
use crate::models::venue::{CreateVenue, PartialVenue, SelectManyFilter, Venue};
use crate::repositories::pool_handler::PoolHandler;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{QueryBuilder, Row};

#[async_trait]
pub trait VenueRepository {
    async fn list_venues(&self, filter: SelectManyFilter) -> Result<Vec<Venue>>;
    async fn get_venue_by_id(&self, location_id: i32) -> Result<Venue, RepositoryError>;
    async fn create_venue(&self, new_location: CreateVenue) -> Result<Venue, RepositoryError>;
    async fn delete_venue(&self, location_id: i32) -> Result<(), RepositoryError>;
    async fn update_venue(
        &self,
        location_id: i32,
        patch_location: PartialVenue,
    ) -> Result<Venue, RepositoryError>;
    async fn list_states(&self) -> Result<Vec<String>>;
    async fn list_cities(&self) -> Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct PgVenueRepository {
    pub pool_handler: PoolHandler,
}

impl PgVenueRepository {
    pub fn new(pool_handler: PoolHandler) -> Self {
        Self { pool_handler }
    }
}

#[async_trait]
impl VenueRepository for PgVenueRepository {
    async fn list_venues(&self, filter: SelectManyFilter) -> Result<Vec<Venue>> {
        let mut query_builder = QueryBuilder::new(
            r#"SELECT 
                    "id",
                    "name",
                    "description",
                    "state",
                    "postal_code",
                    "town",
                    "street_name",
                    "street_number",
                    "address_url"
                FROM "venue" WHERE 1=1"#,
        );

        if let Some(name) = filter.name {
            query_builder.push(r#" AND "name" = "#);
            query_builder.push_bind(name);
        }

        if let Some(state) = filter.state {
            query_builder.push(r#" AND "state" = "#);
            query_builder.push_bind(state);
        }

        if let Some(street_name) = filter.street_name {
            query_builder.push(r#" AND "street_name" = "#);
            query_builder.push_bind(street_name);
        }

        if let Some(postal_code) = filter.postal_code {
            query_builder.push(r#" AND "postal_code" = "#);
            query_builder.push_bind(postal_code);
        }

        if let Some(town) = filter.town {
            query_builder.push(r#" AND "town" = "#);
            query_builder.push_bind(town);
        }

        if let Some(street_number) = filter.street_number {
            query_builder.push(r#" AND "street_number" = "#);
            query_builder.push_bind(street_number);
        }

        let query = query_builder.build();
        let rows = query.fetch_all(self.pool_handler.pool()).await?;

        let data: Result<Vec<Venue>, sqlx::Error> = rows
            .into_iter()
            .map(|row| {
                Ok(Venue {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    state: row.try_get("state")?,
                    postal_code: row.try_get("postal_code")?,
                    town: row.try_get("town")?,
                    street_name: row.try_get("street_name")?,
                    street_number: row.try_get("street_number")?,
                    address_url: row.try_get("address_url")?,
                })
            })
            .collect();

        let data = data?;
        Ok(data)
    }

    async fn get_venue_by_id(&self, venue_id: i32) -> Result<Venue, RepositoryError> {
        let venue = sqlx::query_as!(
            Venue,
            r#"SELECT 
                "id",
                "name",
                "description",
                "state",
                "postal_code",
                "town",
                "street_name",
                "street_number",
                "address_url"
            FROM "venue" WHERE "id" = $1"#,
            venue_id
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(venue) = venue {
            return Ok(venue);
        }

        Err(RepositoryError::NotFound)
    }

    async fn create_venue(&self, new_location: CreateVenue) -> Result<Venue, RepositoryError> {
        let venue = sqlx::query_as!(
            Venue,
            r#"INSERT INTO "venue" (
                "name", "description", "state", "postal_code", "town", "street_name", "street_number", "address_url"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                "id",
                "name",
                "description",
                "state",
                "postal_code",
                "town",
                "street_name",
                "street_number",
                "address_url"
            "#,
            new_location.name,
            new_location.description,
            new_location.state,
            new_location.postal_code,
            new_location.town,
            new_location.street_name,
            new_location.street_number,
            new_location.address_url
        )
            .fetch_one(self.pool_handler.pool())
            .await?;

        Ok(venue)
    }

    async fn delete_venue(&self, location_id: i32) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM "venue" WHERE "id" = $1"#, location_id)
            .execute(self.pool_handler.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }

    async fn update_venue(
        &self,
        venue_id: i32,
        patch_venue: PartialVenue,
    ) -> Result<Venue, RepositoryError> {
        let venue = self.get_venue_by_id(venue_id).await?;

        let state = patch_venue.state.unwrap_or(venue.state);
        let postal_code = patch_venue.postal_code.unwrap_or(venue.postal_code);
        let town = patch_venue.town.unwrap_or(venue.town);
        let street_name = patch_venue.street_name.unwrap_or(venue.street_name);
        let street_number = patch_venue.street_number.unwrap_or(venue.street_number);
        let name = patch_venue.name.unwrap_or(venue.name);
        let description = patch_venue.description.or(venue.description);
        let address_url = patch_venue.address_url.or(venue.address_url);

        let venue = sqlx::query_as!(
            Venue,
            r#"UPDATE "venue" SET
                "state" = $2,
                "postal_code" = $3,
                "town" = $4,
                "street_name" = $5,
                "street_number" = $6,
                "name" = $7,
                "description" = $8,
                "address_url" = $9
            WHERE "id" = $1
            RETURNING 
                "id",
                "name",
                "description",
                "state",
                "postal_code",
                "town",
                "street_name",
                "street_number",
                "address_url"
            "#,
            venue_id,
            state,
            postal_code,
            town,
            street_name,
            street_number,
            name,
            description,
            address_url
        )
        .fetch_optional(self.pool_handler.pool())
        .await?;

        if let Some(venue) = venue {
            return Ok(venue);
        }

        Err(RepositoryError::NotFound)
    }

    async fn list_states(&self) -> Result<Vec<String>> {
        let states = sqlx::query!(r#"SELECT DISTINCT "state" FROM "venue""#)
            .fetch_all(self.pool_handler.pool())
            .await?;
        let states = states.into_iter().map(|row| row.state).collect();
        Ok(states)
    }

    async fn list_cities(&self) -> Result<Vec<String>> {
        let cities = sqlx::query!(r#"SELECT DISTINCT "town" FROM "venue""#)
            .fetch_all(self.pool_handler.pool())
            .await?;
        let cities = cities.into_iter().map(|row| row.town).collect();
        Ok(cities)
    }
}
