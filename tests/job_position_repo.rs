#[cfg(test)]
pub mod job_position_repo_tests {
    use anyhow::Result;
    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::job_position::{
        CreateJobPosition, PartialJobPosition, SalaryCurrency, SelectManyFilter,
    };
    use pv281_giglog::repositories::job_position::JobPositionRepository;
    use pv281_giglog::repositories::job_position::PgJobPositionRepository;
    use pv281_giglog::repositories::pool_handler::PoolHandler;
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test(fixtures("jobs"))]
    async fn test_create_job_position(pool: PgPool) -> Result<()> {
        let mut repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreateJobPosition {
            name: r#"Malomocný s cedulí "Free hugs""#.to_string(),
            description: "Stačí vypadat jako malomocný".to_string(),
            salary: 350.0,
            currency: SalaryCurrency::CZK,
            capacity: 2,
            instructions_html: "Co dělá hudební skladatel Mozart v hrobě? - Rozkládá.".to_string(),
            is_opened_for_registration: false,
            event_id: 1,
            position_category_id: 1,
        };

        let result = repository
            .create_job_position(new.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(result.name, new.name);
        assert_eq!(result.description, Some(new.description));
        assert_eq!(result.salary, new.salary);
        assert_eq!(result.currency, new.currency);
        assert_eq!(result.capacity, new.capacity);
        assert_eq!(result.instructions_html, new.instructions_html);
        assert_eq!(
            result.is_opened_for_registration,
            new.is_opened_for_registration
        );
        assert_eq!(result.event_id, result.event_id);
        assert_eq!(result.position_category_id, result.position_category_id);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("jobs"))]
    async fn test_create_job_positions_should_fail(pool: PgPool) -> Result<()> {
        let mut repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreateJobPosition {
            name: r#"Malomocný s cedulí "Free hugs""#.to_string(),
            description: "Stačí vypadat jako malomocný".to_string(),
            salary: 350.0,
            currency: SalaryCurrency::CZK,
            capacity: 2,
            instructions_html: "Co dělá hudební skladatel Mozart v hrobě? - Rozkládá.".to_string(),
            is_opened_for_registration: false,
            event_id: 2,
            position_category_id: 1,
        };

        let result = repository
            .create_job_position(new.clone())
            .await;

        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("The event has already ended")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("jobs"))]
    async fn test_list_job_positions(pool: PgPool) -> Result<()> {
        let mut repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(pool)));

        let empty = SelectManyFilter {
            event_id: None,
            position_category_id: None,
            salary: None,
            currency: None,
            capacity: None,
            is_opened_for_registration: None,
        };
        let list = repository
            .list_job_positions(empty)
            .await
            .expect("Repository call should succeed");
        assert_eq!(list.len(), 3);

        let filter = SelectManyFilter {
            event_id: None,
            position_category_id: None,
            salary: Some(150.0),
            currency: None,
            capacity: None,
            is_opened_for_registration: Some(true),
        };

        let job = repository
            .list_job_positions(filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(job.len(), 1);

        let unmatchable = SelectManyFilter {
            event_id: None,
            position_category_id: None,
            salary: Some(160.0),
            currency: None,
            capacity: Some(2),
            is_opened_for_registration: None,
        };

        let result = repository
            .list_job_positions(unmatchable)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 0);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("jobs"))]
    async fn test_get_job_position_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(pool)));

        let job = repository
            .get_job_position_by_id(1)
            .await
            .expect("Repository call should succeed");
        assert_eq!(job.name, "Stánek s hotdogy".to_string());
        assert_eq!(job.event_id, 1);
        assert_eq!(job.position_category_id, 1);
        assert_eq!(job.salary, 150.0);
        assert_eq!(job.currency, SalaryCurrency::CZK);
        assert_eq!(job.capacity, 1);
        assert!(job.is_opened_for_registration);
        assert_eq!(
            job.description,
            Some("Prodej hotdogů a dalších rychlých jídel návštěvníkům".to_string())
        );
        assert_eq!(
            job.instructions_html,
            "Co vznikne zkřížením komára a mouchy? - Komouš.".to_string()
        );

        let result = repository.get_job_position_by_id(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("jobs"))]
    async fn test_delete_job_position(pool: PgPool) -> Result<()> {
        let mut repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(pool)));

        repository
            .delete_job_position(1)
            .await
            .expect("Repository call should succeed");

        let result = repository.delete_job_position(1).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("jobs"))]
    async fn test_update_job_position(pool: PgPool) -> Result<()> {
        let mut repository = PgJobPositionRepository::new(PoolHandler::new(Arc::new(pool)));

        let old = repository
            .get_job_position_by_id(2)
            .await
            .expect("Repository call should succeed");

        let new = PartialJobPosition {
            name: Some("New name".to_string()),
            description: Some("Short description for ".to_string()),
            salary: Some(69.69),
            currency: None,
            capacity: None,
            instructions_html: None,
            is_opened_for_registration: None,
            event_id: None,
            position_category_id: None,
        };

        let updated = repository
            .update_job_position(2, new.clone())
            .await
            .expect("Repository call should succeed");
        assert_eq!(updated.name, new.name.unwrap());
        assert_eq!(updated.description, new.description);
        assert_eq!(updated.salary, new.salary.unwrap());
        assert_eq!(updated.currency, old.currency);
        assert_eq!(updated.capacity, old.capacity);
        assert_eq!(updated.instructions_html, old.instructions_html);
        assert_eq!(
            updated.is_opened_for_registration,
            old.is_opened_for_registration
        );
        assert_eq!(updated.event_id, old.event_id);
        assert_eq!(updated.position_category_id, old.position_category_id);

        let empty = PartialJobPosition {
            name: None,
            description: None,
            salary: None,
            currency: None,
            capacity: None,
            instructions_html: None,
            is_opened_for_registration: None,
            event_id: None,
            position_category_id: None,
        };

        let result = repository.update_job_position(999, empty).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
