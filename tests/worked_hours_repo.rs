#[cfg(test)]
pub mod worked_hours_repo_tests {
    use anyhow::Result;
    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::worked_hours::{
        CreateWorkedHours, PartialWorkedHours, SelectManyFilter,
    };
    use pv281_giglog::repositories::pool_handler::PoolHandler;
    use pv281_giglog::repositories::worked_hours::PgWorkedHoursRepository;
    use pv281_giglog::repositories::worked_hours::WorkedHoursRepository;
    use sqlx::types::time::Date;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower_sessions::cookie::time::Month;

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_create_worked_hours(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let partial_new = CreateWorkedHours {
            date: Date::from_calendar_date(2025, Month::January, 2)?,
            hours_worked: 5.5,
            employment_id: 4,
        };

        let new = repository
            .create_worked_hours(partial_new.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(new.id, 4);
        assert_eq!(new.hours_worked, partial_new.hours_worked);
        assert_eq!(new.employment_id, 4);
        assert_eq!(new.date, partial_new.date);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_create_worked_hours_failed_wrong_employment_state(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let worked_new = CreateWorkedHours {
            date: Date::from_calendar_date(2025, Month::January, 2)?,
            hours_worked: 4.0,
            employment_id: 1,
        };

        let result = repository
            .create_worked_hours(worked_new.clone())
            .await;

        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("logging hours job in wrong state")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_create_worked_hours_failed_same_day_same_job(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let worked_new = CreateWorkedHours {
            date: Date::from_calendar_date(2025, Month::January, 2)?,
            hours_worked: 4.0,
            employment_id: 3,
        };

        let result = repository
            .create_worked_hours(worked_new.clone())
            .await;

        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("cannot log worked hours for one job in same day twice")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_create_worked_hours_failed_bad_date(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let worked_new = CreateWorkedHours {
            date: Date::from_calendar_date(2025, Month::January, 4)?,
            hours_worked: 4.0,
            employment_id: 3,
        };

        let result = repository
            .create_worked_hours(worked_new.clone())
            .await;

        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("logging hours date outside of events date")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_list_worked_hours(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let empty_filter = SelectManyFilter {
            date: None,
            hours_worked: None,
            employment_id: None,
        };

        let list_all = repository
            .list_worked_hours(empty_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(list_all.len(), 3);

        let date_filter = SelectManyFilter {
            date: Some(Date::from_calendar_date(2025, Month::January, 2)?),
            hours_worked: None,
            employment_id: None,
        };

        let list_date = repository
            .list_worked_hours(date_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(list_date.len(), 3);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_get_worked_hours_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let log = repository
            .get_worked_hours_by_id(1)
            .await
            .expect("Repository call should succeed");
        assert_eq!(log.id, 1);
        assert_eq!(log.employment_id, 1);
        assert_eq!(log.hours_worked, 5.1);
        assert_eq!(log.date, Date::from_calendar_date(2025, Month::January, 2)?);

        let result = repository.get_worked_hours_by_id(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_delete_worked_hours(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        repository
            .delete_worked_hours(1)
            .await
            .expect("Repository call should succeed");
        repository
            .delete_worked_hours(2)
            .await
            .expect("Repository call should succeed");
        repository
            .delete_worked_hours(3)
            .await
            .expect("Repository call should succeed");

        let result = repository.delete_worked_hours(1).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("worked_hours"))]
    async fn test_update_worked_hours(pool: PgPool) -> Result<()> {
        let mut repository = PgWorkedHoursRepository::new(PoolHandler::new(Arc::new(pool)));

        let old = repository
            .get_worked_hours_by_id(2)
            .await
            .expect("Repository call should succeed");

        let partial = PartialWorkedHours {
            date: None,
            hours_worked: Some(3.5),
            employment_id: None,
        };

        let updated = repository
            .update_worked_hours(2, partial.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(updated.id, 2);
        assert_eq!(updated.date, old.date);
        assert_eq!(updated.hours_worked, partial.hours_worked.unwrap());
        assert_eq!(updated.employment_id, old.employment_id);

        let result = repository.update_worked_hours(999, partial).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
