#[cfg(test)]
pub mod position_category_repo_tests {
    use anyhow::Result;
    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::position_category::{
        CreatePositionCategory, PartialPositionCategory, PositionCategory,
    };
    use pv281_giglog::repositories::pool_handler::PoolHandler;
    use pv281_giglog::repositories::position_category::PgPositionCategoryRepository;
    use pv281_giglog::repositories::position_category::PositionCategoryRepository;
    use sqlx::PgPool;
    use std::sync::Arc;

    #[sqlx::test(fixtures("position_category"))]
    async fn test_create_position_category(pool: PgPool) -> Result<()> {
        let mut repository = PgPositionCategoryRepository::new(PoolHandler::new(Arc::new(pool)));

        let new = CreatePositionCategory {
            name: "Animátor".to_string(),
        };

        let category = repository
            .create_position_category(new.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(category.name, new.name);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("position_category"))]
    async fn test_list_position_category(pool: PgPool) -> Result<()> {
        let mut repository = PgPositionCategoryRepository::new(PoolHandler::new(Arc::new(pool)));

        let category_list = repository
            .list_position_categories()
            .await
            .expect("Repository call should succeed");
        assert_eq!(category_list.len(), 3);
        let expected_list = vec![
            PositionCategory {
                id: 1,
                name: "Obsluha stánků".to_string(),
            },
            PositionCategory {
                id: 2,
                name: "Koordinace parkoviště".to_string(),
            },
            PositionCategory {
                id: 3,
                name: "Technická podpora".to_string(),
            },
        ];
        assert_eq!(category_list, expected_list);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("position_category"))]
    async fn test_get_position_category_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgPositionCategoryRepository::new(PoolHandler::new(Arc::new(pool)));

        let category = repository
            .get_position_category_by_id(2)
            .await
            .expect("Repository call should succeed");
        assert_eq!(category.name, "Koordinace parkoviště".to_string());

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("position_category"))]
    async fn test_delete_position_category(pool: PgPool) -> Result<()> {
        let mut repository = PgPositionCategoryRepository::new(PoolHandler::new(Arc::new(pool)));

        repository
            .delete_position_category(3)
            .await
            .expect("Repository call should succeed");

        let result = repository.delete_position_category(3).await;
        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("position_category"))]
    async fn test_update_position_category(pool: PgPool) -> Result<()> {
        let mut repository = PgPositionCategoryRepository::new(PoolHandler::new(Arc::new(pool)));

        let _old = repository
            .get_position_category_by_id(1)
            .await
            .expect("Repository call should succeed");
        let update = PartialPositionCategory {
            name: Some("Úklid".to_string()),
        };
        let updated = repository
            .update_position_category(1, update.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(updated.name, update.name.clone().unwrap());

        let result = repository.update_position_category(999, update).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
