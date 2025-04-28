use crate::handlers::app::auth::AuthSession;
use axum::extract::State;

use crate::error::AppError;

pub mod get {
    use axum::response::Redirect;

    use crate::app::AppState;

    use super::*;

    pub async fn index(
        _auth_session: AuthSession,
        State(_app_state): State<AppState>,
    ) -> Result<Redirect, AppError> {
        Ok(Redirect::permanent("/events"))
    }
}
