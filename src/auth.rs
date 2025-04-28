use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use core::str;
use password_auth::verify_password;
use serde::Deserialize;
use tokio::task;

use crate::{
    models::user::User,
    repositories::user::{PgUserRepository, UserRepository},
};

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backend {
    user_repository: PgUserRepository,
}

impl Backend {
    pub fn new(user_repository: PgUserRepository) -> Self {
        Self { user_repository }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),

    #[error(transparent)]
    Utf8(#[from] str::Utf8Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = self
            .user_repository
            .get_user_by_username(creds.username)
            .await
            .map_err(anyhow::Error::new)?;

        task::spawn_blocking(|| {
            if verify_password(creds.password, &user.password_hash).is_ok() {
                Ok(Some(user))
            } else {
                Ok(None)
            }
        })
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = self.user_repository.get_user_by_id(*user_id).await.unwrap();
        Ok(Some(user))
    }
}
