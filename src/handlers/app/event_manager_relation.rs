pub mod post {
    use askama::Template;
    use askama_axum::IntoResponse;
    use axum::{
        extract::State,
        response::{Html, Response},
        Form,
    };
    use serde::Deserialize;
    use validator::Validate;

    use crate::{
        app::AppState,
        error::AppError,
        handlers::app::auth::AuthSession,
        models::{
            event_manager_relation::CreateEventManagerRelation,
            user::{self, User, UserRole},
        },
        repositories::{
            event_manager_relation::EventManagerRelationRepository, user::UserRepository,
        },
        templates::EventManagersTemplate,
    };

    #[derive(Deserialize, Validate)]
    pub struct Params {
        event_id: i32,
        manager_id: i32,
    }

    pub async fn event_manager_relation(
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Form<Params>,
    ) -> Result<Response, AppError> {
        let _new_event_manager_relation = app_state
            .event_manager_relation_repository
            .create_relation(CreateEventManagerRelation {
                user_id: params.manager_id,
                event_id: params.event_id,
            })
            .await?;

        let mut managers: Vec<User> = Vec::new();
        let event_manager_relations = app_state
            .event_manager_relation_repository
            .list_event_managers(params.event_id)
            .await?;
        for event_manager_relation in event_manager_relations {
            let manager = app_state
                .user_repository
                .get_user_by_id(event_manager_relation.user_id)
                .await?;
            managers.push(manager);
        }
        let possible_managers = app_state
            .user_repository
            .list_users(user::SelectManyFilter {
                first_name: None,
                last_name: None,
                username: None,
                gender: None,
                role: Some(UserRole::Organizer),
                tax_rate: None,
            })
            .await?;
        let possible_managers = possible_managers
            .into_iter()
            .filter(|possible_manager| {
                !managers
                    .iter()
                    .any(|manager| manager.id == possible_manager.id)
            })
            .collect();

        let template = EventManagersTemplate {
            possible_managers,
            managers,
            event_id: params.event_id,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}

pub mod delete {
    use askama::Template;
    use askama_axum::IntoResponse;
    use axum::{
        extract::{Query, State},
        response::{Html, Response},
    };
    use serde::Deserialize;
    use validator::Validate;

    use crate::{
        app::AppState,
        error::AppError,
        handlers::app::auth::AuthSession,
        models::{
            event_manager_relation::EventManagerRelation,
            user::{self, User, UserRole},
        },
        repositories::{
            event_manager_relation::EventManagerRelationRepository, user::UserRepository,
        },
        templates::EventManagersTemplate,
    };

    #[derive(Deserialize, Validate)]
    pub struct Params {
        event_id: i32,
        manager_id: i32,
    }

    pub async fn event_manager_relation(
        _auth_session: AuthSession,
        State(app_state): State<AppState>,
        params: Query<Params>,
    ) -> Result<Response, AppError> {
        app_state
            .event_manager_relation_repository
            .delete_relation(EventManagerRelation {
                user_id: params.manager_id,
                event_id: params.event_id,
            })
            .await?;

        let possible_managers = app_state
            .user_repository
            .list_users(user::SelectManyFilter {
                first_name: None,
                last_name: None,
                username: None,
                gender: None,
                role: Some(UserRole::Organizer),
                tax_rate: None,
            })
            .await?;
        let mut managers: Vec<User> = Vec::new();
        let event_manager_relations = app_state
            .event_manager_relation_repository
            .list_event_managers(params.event_id)
            .await?;
        for event_manager_relation in event_manager_relations {
            let manager = app_state
                .user_repository
                .get_user_by_id(event_manager_relation.user_id)
                .await?;
            managers.push(manager);
        }

        let template = EventManagersTemplate {
            possible_managers,
            managers,
            event_id: params.event_id,
        };
        let html = template.render().unwrap();
        Ok(Html(html).into_response())
    }
}
