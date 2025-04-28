#[cfg(test)]
pub mod event_manager_relation_repo_tests {
    use anyhow::Result;
    use sqlx::PgPool;
    use std::sync::Arc;
    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::event_manager_relation::CreateEventManagerRelation;
    use pv281_giglog::repositories::event_manager_relation::EventManagerRelationRepository;
    use pv281_giglog::repositories::event_manager_relation::PgEventManagerRelationRepository;
    use pv281_giglog::repositories::pool_handler::PoolHandler;

    #[sqlx::test(fixtures("event_manager_relation"))]
    async fn test_create_event_manager_relation(pool: PgPool) -> Result<()> {
        let mut repository =
            PgEventManagerRelationRepository::new(PoolHandler::new(Arc::new(pool)));

        let new_relation = CreateEventManagerRelation {
            event_id: 1,
            user_id: 3,
        };

        let result = repository
            .create_relation(new_relation.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(result.event_id, 1);
        assert_eq!(result.user_id, 3);

        let result = repository.create_relation(new_relation).await;
        assert!(matches!(result, Err(RepositoryError::GenericError(ref msg)) if msg.contains("The event has already been registered")));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("event_manager_relation"))]
    async fn test_list_event_managers(pool: PgPool) -> Result<()> {
        let mut repository =
            PgEventManagerRelationRepository::new(PoolHandler::new(Arc::new(pool)));

        let result = repository
            .list_event_managers(2)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 2);

        let result = repository
            .list_event_managers(999)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 0);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("event_manager_relation"))]
    async fn test_list_managers_events(pool: PgPool) -> Result<()> {
        let mut repository =
            PgEventManagerRelationRepository::new(PoolHandler::new(Arc::new(pool)));

        let result = repository
            .list_managers_events(3)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 1);

        let result = repository
            .list_managers_events(999)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 0);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("event_manager_relation"))]
    async fn test_delete_relation(pool: PgPool) -> Result<()> {
        let mut repository =
            PgEventManagerRelationRepository::new(PoolHandler::new(Arc::new(pool)));

        let result = repository
            .list_managers_events(3)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 1);

        repository
            .delete_relation(result[0].clone())
            .await
            .expect("Repository call should succeed");

        let result = repository
            .list_managers_events(3)
            .await
            .expect("Repository call should succeed");
        assert_eq!(result.len(), 0);

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
