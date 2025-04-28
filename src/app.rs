use std::{env, sync::Arc};

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use axum::routing::patch;
use axum_login::{login_required, AuthManagerLayerBuilder};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tower_sessions::{cookie::Key, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

use crate::middleware;
use crate::{
    auth::Backend,
    handlers,
    repositories::{
        employment::PgEmploymentRepository, event::PgEventRepository,
        event_manager_relation::PgEventManagerRelationRepository,
        job_position::PgJobPositionRepository, pool_handler::PoolHandler,
        position_category::PgPositionCategoryRepository, user::PgUserRepository,
        venue::PgVenueRepository, worked_hours::PgWorkedHoursRepository,
    },
};

const DEFAULT_HOSTNAME: &str = "0.0.0.0";
const DEFAULT_PORT: &str = "3000";

pub struct App {
    database_url: String,
    server_address: String,
    db_pool: Arc<PgPool>,
}

#[derive(Clone)]
pub struct AppState {
    pub user_repository: PgUserRepository,
    pub employment_repository: PgEmploymentRepository,
    pub event_repository: PgEventRepository,
    pub venue_repository: PgVenueRepository,
    pub job_position_repository: PgJobPositionRepository,
    pub position_category_repository: PgPositionCategoryRepository,
    pub worked_hours_repository: PgWorkedHoursRepository,
    pub event_manager_relation_repository: PgEventManagerRelationRepository,
}

impl App {
    pub async fn new() -> Result<Self> {
        // Load env variables
        if cfg!(debug_assertions) {
            dotenvy::dotenv().expect("Unable to access .env file");
        } else {
            println!("Running in production mode!");
        }

        let server_address = env::var("SERVER_ADDRESS")
            .unwrap_or(format!("{DEFAULT_HOSTNAME}:{DEFAULT_PORT}").to_owned());
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL was not found in .env file.");

        // Create database pool
        let db_pool = Arc::new(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?,
        );

        // Apply migrations
        sqlx::migrate!().run(&*db_pool.clone()).await?;

        Ok(Self {
            database_url,
            server_address,
            db_pool,
        })
    }

    pub async fn serve(&self) -> Result<()> {
        // Setup logging
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        // Initialize global app state
        let app_state = AppState {
            user_repository: PgUserRepository::new(PoolHandler::new(self.db_pool.clone())),
            employment_repository: PgEmploymentRepository::new(PoolHandler::new(
                self.db_pool.clone(),
            )),
            event_repository: PgEventRepository::new(PoolHandler::new(self.db_pool.clone())),
            venue_repository: PgVenueRepository::new(PoolHandler::new(self.db_pool.clone())),
            job_position_repository: PgJobPositionRepository::new(PoolHandler::new(
                self.db_pool.clone(),
            )),
            position_category_repository: PgPositionCategoryRepository::new(PoolHandler::new(
                self.db_pool.clone(),
            )),
            worked_hours_repository: PgWorkedHoursRepository::new(PoolHandler::new(
                self.db_pool.clone(),
            )),
            event_manager_relation_repository: PgEventManagerRelationRepository::new(
                PoolHandler::new(self.db_pool.clone()),
            ),
        };

        // Setup auth
        let session_store = PostgresStore::new(
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&self.database_url)
                .await
                .expect("Can't connect to database!"),
        );
        session_store.migrate().await?;
        let key = Key::generate();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_signed(key);
        let backend = Backend::new(app_state.user_repository.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        // Setup routes
        let app_router = Router::new()
            .route("/admin/users", get(handlers::app::admin::users::get::users).post(handlers::app::admin::users::post::users))
            .route("/admin/user/:user_id", get(handlers::app::admin::user::get::user).delete(handlers::app::admin::user::delete::user))
            .route("/admin/user", patch(handlers::app::admin::user::patch::user).post(handlers::app::admin::user::post::register))
            .route("/admin/user/create", get(handlers::app::admin::user::get_create_template::user))
            .route(
                "/admin/events",
                get(handlers::app::admin::events::get::events),
            )
            .route("/admin/jobs", get(handlers::app::admin::jobs::get::jobs))
            .route_layer(axum::middleware::from_fn_with_state(app_state.clone(), middleware::global::check_admin))
            .route("/employments", get(handlers::app::employments::get::employments).post(handlers::app::employments::post::employments))
            .route("/employments/action", post(handlers::partials::base::main::employments::employments_action::post::action))
            .route("/employees", get(handlers::app::employees::get::employees))
            .route(
                "/employees/:employee_id",
                get(handlers::app::employees::employee::get::employee),
            )
            .route(
                "/manage/events/:event_id",
                get(handlers::app::events::event::manage::get::manage),
            )
            .route(
                "/manage/events",
                get(handlers::app::events::manage::get::manage),
            )
            .route(
                "/manage/jobs",
                get(handlers::app::jobs::manage::get::manage).patch(handlers::app::jobs::job::manage::patch::manage),
            )
            .route(
                "/manage/jobs/:job_id",
                get(handlers::app::jobs::job::manage::get::manage).delete(handlers::app::jobs::job::manage::delete::manage),
            )
            .route(
                "/manage/venues",
                get(handlers::app::venues::manage::get::manage).patch(handlers::app::venues::venue::manage::patch::manage),
            )
            .route(
                "/manage/venues/:venue_id",
                get(handlers::app::venues::venue::manage::get::manage).delete(handlers::app::venues::venue::manage::delete::manage),
            )
            .route(
                "/create/venues",
                get(handlers::app::venues::venue::create::get::create).post(handlers::app::venues::venue::create::post::create),
            )
            .route(
                "/create/events",
                get(handlers::app::events::event::create::get::create),
            )
            .route(
                "/create/jobs",
                get(handlers::app::jobs::create::get::create).post(handlers::app::jobs::create::post::create),
            )
            .route_layer(axum::middleware::from_fn_with_state(app_state.clone(), middleware::global::check_organizer))
            .route(
                "/partials/base/main/attendance/attendance-log",
                get(handlers::partials::base::main::attendance::attendance_log::get::attendance_log).patch(handlers::partials::base::main::attendance::attendance_log::patch::attendance_log)
            )
            .route(
                "/attendance",
                get(handlers::app::attendance::get::attendance),
            )   
            .route(
                "/partials/base/main/attendance/attendance-job-options",
                get(
                    handlers::partials::base::main::attendance::attendance_job_options::get::attendance_job_options,
                ),
            )
            .route("/jobs", get(handlers::app::jobs::get::jobs).post(handlers::app::jobs::post::jobs))
            .route("/partials/base/main/events/event/job-state/:job_id", post(handlers::partials::base::main::events::event::job_state::post::job_state))
            .route("/events", post(handlers::app::events::post::events).patch(handlers::app::events::patch::events))
            .route(
                "/employment",
                post(handlers::app::employment::post::employment).delete(handlers::app::employment::delete::employment),
            )
            .route("/event-manager-relation", post(handlers::app::event_manager_relation::post::event_manager_relation).delete(handlers::app::event_manager_relation::delete::event_manager_relation))
            .route("/settings/details", get(handlers::app::settings::details::get::details).patch(handlers::app::settings::details::patch::details))
            .route("/settings/password", get(handlers::app::settings::password::get::password).patch(handlers::app::settings::password::patch::password))
            
            .route("/logout", get(handlers::app::auth::get::logout))
            .route_layer(login_required!(Backend, login_url = "/login"))
            .route("/", get(handlers::app::index::get::index))
            .route(
                "/login",
                get(handlers::app::auth::get::login).post(handlers::app::auth::post::login),
            )
            .route("/register", get(handlers::app::auth::get::register).post(handlers::app::auth::post::register))
            .route("/events", get(handlers::app::events::get::events))
            .route(
                "/events/:event_id",
                get(handlers::app::events::event::get::event).delete(handlers::app::events::event::delete::event),
            )
            .route("/partials/base/main/events/events-content", get(handlers::partials::base::main::events::events_content::get::events_content))
            .fallback(handlers::app::page_not_found::page_not_found)
            .with_state(app_state)
            .nest_service("/public", ServeDir::new("public"))
            .layer(TraceLayer::new_for_http())
            .layer(auth_layer);

        // Bind server
        let listener = TcpListener::bind(&self.server_address).await?;
        println!(
            "Listening on http://{}",
            listener
                .local_addr()
                .expect("Local address should be bound!")
        );

        // Serve
        axum::serve(listener, app_router).await?;
        Ok(())
    }
}
