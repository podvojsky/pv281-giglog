#[cfg(test)]
pub mod event_repo_tests {
    use anyhow::Result;
    use sqlx::types::time::Date;
    use sqlx::PgPool;
    use std::sync::Arc;

    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::event::{CreateEvent, PartialEvent, SelectManyFilter};
    use pv281_giglog::repositories::event::EventRepository;
    use pv281_giglog::repositories::event::PgEventRepository;
    use pv281_giglog::repositories::pool_handler::PoolHandler;
    use tower_sessions::cookie::time::Month;

    #[sqlx::test(fixtures("events"))]
    async fn test_create_event(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreateEvent {
            name: "Superfest <3".to_string(),
            date_start: Date::from_calendar_date(2025, Month::May, 13)?,
            date_end: Date::from_calendar_date(2025, Month::May, 14)?,
            img_url: "https://url.com".to_string(),
            description: "Simple description".to_string(),
            is_draft: true,
            venue_id: 1,
            owner_id: 1,
        };

        let event = repository
            .create_event(new.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(event.name, new.name);
        assert_eq!(event.date_start, new.date_start);
        assert_eq!(event.date_end, new.date_end);
        assert_eq!(event.img_url, new.img_url);
        assert_eq!(event.description, event.description);
        assert_eq!(event.is_draft, event.is_draft);
        assert_eq!(event.venue_id, event.venue_id);
        assert_eq!(event.owner_id, event.venue_id);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_list_events(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let empty = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: None,
        };

        let all = repository
            .list_events(empty)
            .await
            .expect("Repository call should succeed");
        assert_eq!(all.len(), 4);

        let filter = SelectManyFilter {
            date_from: Some(Date::from_calendar_date(2025, Month::February, 1)?),
            date_to: Some(Date::from_calendar_date(2025, Month::December, 31)?),
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: None,
        };

        let events = repository
            .list_events(filter)
            .await
            .expect("Repository call should succeed");

        assert_eq!(events.len(), 3);

        let names = vec!["Rocky in 2025".to_string(), "Feets for love".to_string(), "Berlin Music Fest".to_string()];
        let event_names: Vec<String> = events.iter().map(|event| event.name.clone()).collect();
        assert_eq!(names, event_names);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_event_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let event = repository
            .get_event_by_id(2)
            .await
            .expect("Repository call should succeed");

        assert_eq!(event.id, 2);
        assert_eq!(event.name, "Rocky in 2025");
        assert_eq!(
            event.date_start,
            Date::from_calendar_date(2025, Month::May, 25)?
        );
        assert_eq!(
            event.date_end,
            Date::from_calendar_date(2025, Month::May, 25)?
        );
        assert_eq!(event.img_url, "https://cdn.siteone.io/srv.siteone.cz/imgproxy/LhCs_LjiIr027zmUTqCgc-JgrB4-Dx33eI3QWyD0xoI/w:860/h:740/rt:fill/g:no:0:0/f:avif/q:70/aHR0cHM6Ly93d3cubmVrZGVuZWNvLmN6Ly9jbXMtYXNzZXRzL3JvY2staW4tMjAyNV8yMDI0LTExLTA1LTA3MzAwNl9vc3pxLmpwZw.avif");
        assert_eq!(event.description, Some("Zažijte to nejlepší z domácí rockové scény v jeden den na jednom pódiu přímo u vás! Ve vašem městě, ve vašem amfiteátru se vystřídají zvučná jména, s důrazem na profesionální zázemí, špičkovou techniku a maximální komfort pro návštěvníky.".to_string()));
        assert!(!event.is_draft);
        assert_eq!(event.venue_id, 1);
        assert_eq!(event.owner_id, 1);

        let result = repository.get_event_by_id(999).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_delete_event(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let empty_filter = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: None,
        };

        let all = repository
            .list_events(empty_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(all.len(), 4);
        repository
            .delete_event(1)
            .await
            .expect("Repository call should succeed");

        let empty_filter = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: None,
        };

        let all = repository
            .list_events(empty_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(all.len(), 3);

        let result = repository.delete_event(1).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_update_event(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let old = repository
            .get_event_by_id(1)
            .await
            .expect("Repository call should succeed");

        let to_update = PartialEvent {
            name: Some("Traunterberg".to_string()),
            date_start: None,
            date_end: None,
            img_url: None,
            description: Some("Superfest idk".to_string()),
            is_draft: Some(true),
            venue_id: None,
            owner_id: None,
        };

        let updated = repository
            .update_event(1, to_update.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(old.id, updated.id);
        assert_eq!(to_update.name.clone().unwrap(), updated.name);
        assert_eq!(old.date_start, updated.date_start);
        assert_eq!(old.date_end, updated.date_end);
        assert_eq!(old.img_url, updated.img_url);
        assert_eq!(to_update.description.clone(), updated.description);
        assert_eq!(to_update.is_draft.unwrap(), updated.is_draft);
        assert_eq!(old.venue_id, updated.venue_id);
        assert_eq!(old.owner_id, updated.owner_id);

        let result = repository.update_event(999, to_update).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_filter_event_by_name(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let mut filter = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: Some(String::from("fest")),
        };

        let some = repository
            .list_events(filter.clone())
            .await
            .expect("Repository call should succeed");
        assert_eq!(some.len(), 2);

        filter.name = Some(String::from("festival"));

        let none = repository
            .list_events(filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(none.len(), 0);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_filter_event_by_venue(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let mut filter = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: None,
            venue_id: None,
            owner_id: None,
            city: Some(String::from("Plzeň")),
            state: None,
            name: None,
        };

        let some = repository
            .list_events(filter.clone())
            .await
            .expect("Repository call should succeed");
        assert_eq!(some.len(), 1);

        filter.city = None;
        filter.state = Some(String::from("Germany"));

        let some = repository
            .list_events(filter.clone())
            .await
            .expect("Repository call should succeed");
        assert_eq!(some.len(), 1);

        filter.state = Some(String::from("Česká republika"));

        let some = repository
            .list_events(filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(some.len(), 3);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("events"))]
    async fn test_filter_event_by_draft(pool: PgPool) -> Result<()> {
        let mut repository = PgEventRepository::new(PoolHandler::new(Arc::new(pool)));

        let filter = SelectManyFilter {
            date_from: None,
            date_to: None,
            is_draft: Some(true),
            venue_id: None,
            owner_id: None,
            city: None,
            state: None,
            name: None,
        };

        let some = repository
            .list_events(filter.clone())
            .await
            .expect("Repository call should succeed");
        assert_eq!(some.len(), 1);

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
