#[cfg(test)]
pub mod employment_repo_tests {
    use anyhow::Result;
    use sqlx::PgPool;
    use std::sync::Arc;

    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::employment::{
        CreateEmployment, EmploymentState, PartialEmployment, SelectManyFilter,
    };
    use pv281_giglog::repositories::employment::EmploymentRepository;
    use pv281_giglog::repositories::employment::PgEmploymentRepository;
    use pv281_giglog::repositories::pool_handler::PoolHandler;

    #[sqlx::test(fixtures("employment"))]
    async fn test_create_employment(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreateEmployment {
            user_id: 4,
            position_id: 1,
            rating: 5,
            state: EmploymentState::Pending,
        };

        let new_employment = repository
            .create_employment(new.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(new_employment.user_id, new_employment.user_id);
        assert_eq!(new_employment.position_id, new_employment.position_id);
        assert_eq!(new_employment.rating, new_employment.rating);
        assert_eq!(new_employment.state, new_employment.state);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_create_employment_fail_exists(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreateEmployment {
            user_id: 3,
            position_id: 1,
            rating: 5,
            state: EmploymentState::Pending,
        };

        let result = repository
            .create_employment(new.clone())
            .await;

        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("The event has already been registered")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_create_employment_fail_event_after_end(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreateEmployment {
            user_id: 3,
            position_id: 2,
            rating: 5,
            state: EmploymentState::Pending,
        };

        let result = repository
            .create_employment(new.clone())
            .await;

        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("The event has already ended")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_create_employment_fail_full_job_position(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let mut new = CreateEmployment {
            user_id: 4,
            position_id: 1,
            rating: 5,
            state: EmploymentState::Accepted,
        };

        let result = repository
            .create_employment(new.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(result.user_id, new.user_id);
        assert_eq!(result.position_id, new.position_id);
        assert_eq!(result.rating, new.rating);
        assert_eq!(result.state, new.state);

        new.user_id = 5;

        let created = repository
            .create_employment(new.clone())
            .await;

        assert!(matches!(created, Err(RepositoryError::GenericError(ref msg)) if msg.contains("Job position is already full")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_list_employment(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let empty_filter = SelectManyFilter {
            user_id: None,
            position_id: None,
            rating: None,
            state: None,
        };

        let list_all = repository
            .list_employment(empty_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(list_all.len(), 3);

        let accepted_filter = SelectManyFilter {
            user_id: None,
            position_id: None,
            rating: None,
            state: Some(EmploymentState::Accepted),
        };

        let list = repository
            .list_employment(accepted_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(list.len(), 1);

        let user_filter = SelectManyFilter {
            user_id: Some(2),
            position_id: None,
            rating: None,
            state: None,
        };

        let list = repository
            .list_employment(user_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(list.len(), 1);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_get_employment_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let existing = repository
            .get_employment_by_id(1)
            .await
            .expect("Repository call should succeed");
        assert_eq!(existing.user_id, 1);
        assert_eq!(existing.position_id, 1);
        assert_eq!(existing.rating, 8);
        assert_eq!(existing.state, EmploymentState::Pending);

        let result = repository.get_employment_by_id(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_delete_employment(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        repository
            .delete_employment(1)
            .await
            .expect("Repository call should succeed");
        repository
            .delete_employment(2)
            .await
            .expect("Repository call should succeed");
        repository
            .delete_employment(3)
            .await
            .expect("Repository call should succeed");

        let result = repository.get_employment_by_id(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("employment"))]
    async fn test_(pool: PgPool) -> Result<()> {
        let mut repository = PgEmploymentRepository::new(PoolHandler::new(Arc::new(pool)));

        let old = repository
            .get_employment_by_id(2)
            .await
            .expect("Repository call should succeed");

        let partial = PartialEmployment {
            user_id: Some(4),
            position_id: None,
            rating: Some(10),
            state: None,
        };

        let updated = repository
            .update_employment(2, partial.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(partial.user_id.unwrap(), updated.user_id);
        assert_eq!(old.position_id, updated.position_id);
        assert_eq!(partial.rating.unwrap(), updated.rating);
        assert_eq!(old.state, updated.state);

        let result = repository.update_employment(999, partial).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
