#[cfg(test)]
pub mod user_repo_tests {
    use anyhow::Result;
    use sqlx::types::time::Date;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower_sessions::cookie::time::Month;

    use pv281_giglog::error::RepositoryError;
    use pv281_giglog::models::user::{CreateUser, Gender, PartialUser, SelectManyFilter, UserRole};
    use pv281_giglog::repositories::pool_handler::PoolHandler;
    use pv281_giglog::repositories::user::{PgUserRepository, UserRepository};

    #[sqlx::test(fixtures("users"))]
    async fn test_create_user(pool: PgPool) -> Result<()> {
        let mut repository = PgUserRepository::new(PoolHandler::new(Arc::new(pool)));

        let new_user = CreateUser {
            first_name: "Libor".to_string(),
            last_name: "Sobotka".to_string(),
            username: "Lisko".to_string(),
            gender: Gender::Male,
            birth_date: Date::from_calendar_date(2000, Month::March, 13)?,
            email: "Lisko@gmail.com".to_string(),
            phone: "+420777666123".to_string(),
            password_hash: "hash".to_string(),
            tax_rate: 0.25,
            role: UserRole::Employee,
            avatar_url: Some("https://www.url.com".to_string()),
        };

        let created = repository
            .create_user(new_user.clone())
            .await
            .expect("Repository call should succeed");

        assert!(created.id >= 0);
        assert_eq!(created.first_name, new_user.first_name);
        assert_eq!(created.last_name, new_user.last_name);
        assert_eq!(created.username, new_user.username);
        assert_eq!(created.gender, new_user.gender);
        assert_eq!(created.birth_date, new_user.birth_date);
        assert_eq!(created.email, new_user.email);
        assert_eq!(created.phone, new_user.phone);
        assert_eq!(created.password_hash, new_user.password_hash);
        assert_eq!(created.tax_rate, new_user.tax_rate);
        assert_eq!(created.role, new_user.role);

        let existing_username = CreateUser {
            first_name: "".to_string(),
            last_name: "".to_string(),
            username: "Lisko".to_string(),
            gender: Gender::Male,
            birth_date: Date::from_calendar_date(2002, Month::December, 16)?,
            email: "".to_string(),
            phone: "".to_string(),
            password_hash: "".to_string(),
            role: UserRole::Employee,
            tax_rate: 0.1,
            avatar_url: None,
        };

        let result = repository.create_user(existing_username).await;
        assert!(result.is_err());

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("users"))]
    async fn test_list_users(pool: PgPool) -> Result<()> {
        let mut repository = PgUserRepository::new(PoolHandler::new(Arc::new(pool)));

        let user_filter = SelectManyFilter {
            username: None,
            first_name: None,
            last_name: None,
            gender: None,
            role: None,
            tax_rate: None,
        };

        let users = repository
            .list_users(user_filter)
            .await
            .expect("Repository call should succeed");

        assert_eq!(users.len(), 4);
        let usernames = vec![
            "pepe232".to_string(),
            "brember".to_string(),
            "lasicak".to_string(),
            "fousek".to_string(),
        ];
        let users_username: Vec<String> = users.iter().map(|user| user.username.clone()).collect();

        assert_eq!(usernames, users_username);

        let user_filter = SelectManyFilter {
            username: None,
            first_name: None,
            last_name: None,
            gender: Some(Gender::Male),
            role: None,
            tax_rate: None,
        };

        let users = repository
            .list_users(user_filter)
            .await
            .expect("Repository call should succeed");
        assert_eq!(users.len(), 2);
        let usernames = vec!["brember".to_string(), "fousek".to_string()];
        let users_username: Vec<String> = users.iter().map(|user| user.username.clone()).collect();

        assert_eq!(usernames, users_username);

        let user_filter = SelectManyFilter {
            username: None,
            first_name: None,
            last_name: None,
            gender: Some(Gender::Female),
            role: Some(UserRole::Admin),
            tax_rate: None,
        };

        let users = repository
            .list_users(user_filter)
            .await
            .expect("Repository call should succeed");

        assert_eq!(users.len(), 2);
        let usernames = vec!["pepe232".to_string(), "lasicak".to_string()];
        let users_username: Vec<String> = users.iter().map(|user| user.username.clone()).collect();

        assert_eq!(usernames, users_username);

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("users"))]
    async fn test_get_user_by_id(pool: PgPool) -> Result<()> {
        let mut repository = PgUserRepository::new(PoolHandler::new(Arc::new(pool)));

        let user = repository
            .get_user_by_id(1)
            .await
            .expect("Repository call should succeed");

        assert_eq!(user.id, 1);
        assert_eq!(user.first_name, "Josefka");
        assert_eq!(user.last_name, "Buba");
        assert_eq!(user.username, "pepe232");
        assert_eq!(user.gender, Gender::Female);
        assert_eq!(
            user.birth_date,
            Date::from_calendar_date(2001, Month::April, 11)?
        );
        assert_eq!(user.email, "joko@nba.com");
        assert_eq!(user.phone, "7151703730");
        assert_eq!(user.tax_rate, 0.15);
        assert_eq!(user.role, UserRole::Admin);

        let result = repository.get_user_by_id(999).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("users"))]
    async fn test_get_user_username(pool: PgPool) -> Result<()> {
        let mut repository = PgUserRepository::new(PoolHandler::new(Arc::new(pool)));

        let user = repository
            .get_user_by_username("brember".to_string())
            .await
            .expect("Repository call should succeed");

        assert_eq!(user.id, 2);
        assert_eq!(user.first_name, "Radek");
        assert_eq!(user.last_name, "Srejch");
        assert_eq!(user.username, "brember");
        assert_eq!(user.gender, Gender::Male);
        assert_eq!(
            user.birth_date,
            Date::from_calendar_date(2000, Month::May, 12)?
        );
        assert_eq!(user.email, "brember@mail.com");
        assert_eq!(user.phone, "2212605075");
        assert_eq!(user.tax_rate, 0.15);
        assert_eq!(user.role, UserRole::Employee);

        let result = repository
            .get_user_by_username("raketaMarketa".to_string())
            .await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("users"))]
    async fn test_delete_user(pool: PgPool) -> Result<()> {
        let mut repository = PgUserRepository::new(PoolHandler::new(Arc::new(pool)));

        let user = repository.get_user_by_id(1).await;

        assert!(user.is_ok());

        repository.delete_user(1).await?;
        let result = repository.get_user_by_id(1).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }

    #[sqlx::test(fixtures("users"))]
    async fn test_update_user(pool: PgPool) -> Result<()> {
        let mut repository = PgUserRepository::new(PoolHandler::new(Arc::new(pool)));

        let old_user = repository
            .get_user_by_id(4)
            .await
            .expect("Repository call should succeed");

        let partial = PartialUser {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            username: Some("IamJonny".to_string()),
            gender: None,
            birth_date: None,
            email: None,
            phone: None,
            password_hash: None,
            role: None,
            tax_rate: None,
            avatar_url: None,
        };

        let updated_user = repository
            .update_user(4, partial.clone())
            .await
            .expect("Repository call should succeed");

        assert_eq!(4, updated_user.id);
        assert_eq!(partial.first_name.unwrap(), updated_user.first_name);
        assert_eq!(partial.last_name.unwrap(), updated_user.last_name);
        assert_eq!(partial.username.unwrap(), updated_user.username);
        assert_eq!(old_user.gender, updated_user.gender);
        assert_eq!(old_user.birth_date, updated_user.birth_date);
        assert_eq!(old_user.email, updated_user.email);
        assert_eq!(old_user.phone, updated_user.phone);
        assert_eq!(old_user.password_hash, updated_user.password_hash);
        assert_eq!(old_user.role, updated_user.role);
        assert_eq!(old_user.tax_rate, updated_user.tax_rate);

        let empty_partial = PartialUser {
            first_name: None,
            last_name: None,
            username: None,
            gender: None,
            birth_date: None,
            email: None,
            phone: None,
            password_hash: None,
            role: None,
            tax_rate: None,
            avatar_url: None,
        };

        let result = repository.update_user(999, empty_partial).await;

        assert!(matches!(result, Err(RepositoryError::NotFound)));

        repository.pool_handler.disconnect().await;
        Ok(())
    }
}
