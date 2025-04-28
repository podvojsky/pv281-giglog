#[cfg(test)]
pub mod venue_repo_tests {
    use anyhow::Result;
    use sqlx::PgPool;
    use std::sync::Arc;

    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::venue::{CreateVenue, PartialVenue, SelectManyFilter};
    use pv281_giglog::repositories::pool_handler::PoolHandler;
    use pv281_giglog::repositories::venue::PgVenueRepository;
    use pv281_giglog::repositories::venue::VenueRepository;

    #[sqlx::test(fixtures("venues"))]
    async fn test_create_venue(pool: PgPool) -> Result<()> {
        let mut repository = PgVenueRepository::new(PoolHandler::new(Arc::new(pool)));

        let new_venue = CreateVenue {
            name: "Pražský hrad".to_string(),
            description: Some("Skvělej bejvák".to_string()),
            state: "Česká republika".to_string(),
            postal_code: "674 55".to_string(),
            town: "Praha".to_string(),
            street_name: "Netusim".to_string(),
            street_number: "69".to_string(),
            address_url: Some("https://www.urlrulrrefsd.cz".to_string()),
        };

        let result = repository
            .create_venue(new_venue.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(new_venue.name, result.name);
        assert_eq!(new_venue.description, result.description);
        assert_eq!(new_venue.state, result.state);
        assert_eq!(new_venue.postal_code, result.postal_code);
        assert_eq!(new_venue.town, result.town);
        assert_eq!(new_venue.street_name, result.street_name);
        assert_eq!(new_venue.street_number, result.street_number);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("venues"))]
    async fn test_list_venues(pool: PgPool) -> Result<()> {
        let mut repository = PgVenueRepository::new(PoolHandler::new(Arc::new(pool)));

        let empty_filter = SelectManyFilter {
            name: None,
            description: None,
            state: None,
            postal_code: None,
            town: None,
            street_name: None,
            street_number: None,
        };

        let result = repository
            .list_venues(empty_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 4);

        let names = vec![
            "Plzeň Plaza".to_string(),
            "Brněnské výstaviště".to_string(),
            "Letiště Hradec Králové".to_string(),
            "Katedrála".to_string(),
        ];
        let venue_names: Vec<String> = result.iter().map(|venue| venue.name.clone()).collect();
        assert_eq!(names, venue_names);

        let town_filter = SelectManyFilter {
            name: None,
            description: None,
            state: None,
            postal_code: None,
            town: Some("Plzeň".to_string()),
            street_name: None,
            street_number: None,
        };
        let venue = repository
            .list_venues(town_filter)
            .await
            .expect("Repository call should succeed");
        let venue_names: Vec<String> = venue.iter().map(|venue| venue.name.clone()).collect();
        let names = vec!["Plzeň Plaza".to_string(), "Katedrála".to_string()];
        assert_eq!(names, venue_names);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("venues"))]
    async fn test_venue_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgVenueRepository::new(PoolHandler::new(Arc::new(pool)));

        let venue = repository
            .get_venue_by_id(4)
            .await
            .expect("Repository call should succeed");
        assert_eq!(venue.name, "Katedrála");
        assert_eq!(venue.state, "Česká republika");
        assert_eq!(venue.postal_code, "301 00");
        assert_eq!(venue.town, "Plzeň");
        assert_eq!(venue.street_name, "Neew");
        assert_eq!(venue.street_number, "33");
        assert_eq!(venue.description, None);

        let result = repository.get_venue_by_id(999).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("venues"))]
    async fn test_delete_venue(pool: PgPool) -> Result<()> {
        let mut repository = PgVenueRepository::new(PoolHandler::new(Arc::new(pool)));

        let result = repository.delete_venue(1).await;
        assert!(result.is_ok());

        let result = repository.get_venue_by_id(999).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("venues"))]
    async fn test_update_venue(pool: PgPool) -> Result<()> {
        let mut repository = PgVenueRepository::new(PoolHandler::new(Arc::new(pool)));

        let old_venue = repository
            .get_venue_by_id(2)
            .await
            .expect("Repository call should succeed");

        let partial = PartialVenue {
            name: Some("Updated".to_string()),
            description: None,
            state: None,
            postal_code: None,
            town: Some("Bradavice".to_string()),
            street_name: None,
            street_number: Some("69".to_string()),
            address_url: None,
        };

        let updated = repository
            .update_venue(2, partial.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(updated.id, 2);
        assert_eq!(updated.name, partial.name.unwrap());
        assert_eq!(updated.description, old_venue.description);
        assert_eq!(updated.state, old_venue.state);
        assert_eq!(updated.postal_code, old_venue.postal_code);
        assert_eq!(updated.town, partial.town.unwrap());
        assert_eq!(updated.street_name, old_venue.street_name);
        assert_eq!(updated.street_number, partial.street_number.unwrap());

        let empty = PartialVenue {
            name: None,
            description: None,
            state: None,
            postal_code: None,
            town: None,
            street_name: None,
            street_number: None,
            address_url: None,
        };
        let result = repository.update_venue(999, empty).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
