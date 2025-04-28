use std::collections::BTreeMap;

use askama::Template;
use sqlx::types::time::Date;
use validator::{ValidationErrors, ValidationErrorsKind};

use crate::models::employment::Employment;
use crate::models::job_position::{JobPosition, JobPositionViewModel};
use crate::models::position_category::PositionCategory;
use crate::models::venue::Venue;
use crate::models::worked_hours::WorkedHours;
use crate::view_models::employments::EmploymentViewModel;
use crate::view_models::event::{EventDetailViewModel, EventViewModel, ManageEventViewModel};
use crate::view_models::jobs::{
    ManageJobEmployeeViewModel, ManageJobPositionViewModel, ManageJobPositionsViewModel,
    PastJobsViewModel,
};
use crate::view_models::my_jobs::{JobSummary, MyJobsViewModel};
use crate::view_models::user::UserViewModel;
use crate::{
    handlers::app::auth::AuthSession,
    models::{
        employment::EmploymentState,
        event::Event,
        user::{Gender, User, UserRole},
    },
};

pub enum ActiveRoute {
    Events,
    MyJobs,
    Attendance,
    Employees,
    Manage,
    AdminPanel,
    Employments,
}

pub enum ToastType {
    Success,
    Error,
}

#[derive(Template)]
#[template(path = "views/base/index.html")]
pub struct IndexTemplate {
    #[allow(unused)]
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "views/base/auth/login.html")]
pub struct LoginViewTemplate {
    #[allow(unused)]
    pub session: AuthSession,
    pub next: Option<String>,
}

#[derive(Template)]
#[template(path = "views/base/auth/register.html")]
pub struct RegisterTemplate {}

#[derive(Template)]
#[template(path = "views/base/protected.html")]
pub struct ProtectedTemplate<'a> {
    pub username: &'a str,
}

#[derive(Template)]
#[template(path = "views/base/main/events.html")]
pub struct EventsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub future_events: Vec<EventViewModel>,
    pub past_events: Vec<EventViewModel>,
    pub states: Vec<String>,
    pub cities: Vec<String>,
}

#[derive(Template)]
#[template(path = "htmx/base/main/events/events_content.html")]
pub struct EventsContentTemplate {
    #[allow(unused)]
    pub session: AuthSession,
    pub future_events: Vec<EventViewModel>,
    pub past_events: Vec<EventViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/manage_events.html")]
pub struct ManageEventsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub events: Vec<EventViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/manage_jobs.html")]
pub struct ManageJobsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub jobs: Vec<ManageJobPositionsViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/create_job.html")]
pub struct CreateJobTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub events: Vec<Event>,
    pub job_categories: Vec<PositionCategory>,
}

#[derive(Template)]
#[template(path = "views/base/main/manage_job.html")]
pub struct ManageJobTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub events: Vec<Event>,
    pub job_categories: Vec<PositionCategory>,
    pub job: ManageJobPositionViewModel,
    pub possible_employees: Vec<User>,
    pub employees: Vec<ManageJobEmployeeViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/manage_venues.html")]
pub struct ManageVenuesTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub venues: Vec<Venue>,
}

#[derive(Template)]
#[template(path = "views/base/main/manage_venue.html")]
pub struct ManageVenueTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub venue: Venue,
}

#[derive(Template)]
#[template(path = "views/base/main/create_venue.html")]
pub struct CreateVenueTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
}

#[derive(Template)]
#[template(path = "views/base/main/event.html")]
pub struct EventTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub event: EventDetailViewModel,
    pub is_in_past: bool,
}

#[derive(Template)]
#[template(path = "htmx/base/main/events/event/job_state.html")]
pub struct JobStateTemplate {
    pub session: AuthSession,
    pub job: JobPositionViewModel,
    pub is_in_past: bool,
}

#[derive(Template)]
#[template(path = "views/base/main/manage_event.html")]
pub struct ManageEventTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub event: ManageEventViewModel,
    pub venues: Vec<Venue>,
    pub possible_managers: Vec<User>,
    pub managers: Vec<User>,
}

#[derive(Template)]
#[template(path = "partials/manage_event_managers.html")]
pub struct EventManagersTemplate {
    pub possible_managers: Vec<User>,
    pub managers: Vec<User>,
    pub event_id: i32,
}

#[derive(Template)]
#[template(path = "partials/manage_job_employees.html")]
pub struct JobEmployeesTemplate {
    pub possible_employees: Vec<User>,
    pub employees: Vec<ManageJobEmployeeViewModel>,
    #[allow(unused)]
    pub job_id: i32,
}

#[derive(Template)]
#[template(path = "views/base/main/create_event.html")]
pub struct CreateEventTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub venues: Vec<Venue>,
}

#[derive(Template)]
#[template(path = "views/base/main/employees.html")]
pub struct EmployeesTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub employees: Vec<UserViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/employee.html")]
pub struct EmployeeTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub employee: UserViewModel,
    pub past_jobs: Vec<PastJobsViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/user.html")]
pub struct UserTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub user: User,
}

#[derive(Template)]
#[template(path = "views/base/main/attendance.html")]
pub struct AttendanceTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub events: Vec<Event>,
    // pub date_range: Vec<Date>,
}

#[derive(Template)]
#[template(path = "htmx/base/main/attendance/attendance_log.html")]
pub struct AttendanceLogTemplate {
    #[allow(unused)]
    pub session: AuthSession,
    pub attendance_log: BTreeMap<Date, Option<WorkedHours>>,
    pub employment: Employment,
}

#[derive(Template)]
#[template(path = "htmx/base/main/attendance/hours_worked_input.html")]
pub struct HoursWorkedInputTemplate {
    #[allow(unused)]
    pub session: AuthSession,
    pub worked_hours: WorkedHours,
}

#[derive(Template)]
#[template(path = "htmx/base/main/attendance/attendance_job_options.html")]
pub struct AttendanceJobOptionsTemplate {
    pub jobs: Vec<JobPosition>,
}

#[derive(Template)]
#[template(path = "views/base/404.html")]
pub struct PageNotFoundTemplate {}

#[derive(Template)]
#[template(path = "views/base/401.html")]
pub struct UnauthorizedTemplate {}

#[derive(Template)]
#[template(path = "views/base/main/settings/details.html")]
pub struct SettingsDetailsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
}

#[derive(Template)]
#[template(path = "views/base/main/settings/password.html")]
pub struct SettingsPasswordTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
}

#[derive(Template)]
#[template(path = "views/base/main/my_jobs.html")]
pub struct JobsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub my_jobs: Vec<MyJobsViewModel>,
    pub employment_states: Vec<EmploymentState>,
    pub job_summary: JobSummary,
}

#[derive(Template)]
#[template(path = "views/base/main/my_jobs_table_body.html")]
pub struct JobsTableTemplate {
    pub my_jobs: Vec<MyJobsViewModel>,
    pub job_summary: JobSummary,
}

#[derive(Template)]
#[template(path = "partials/toast.html")]
pub struct ToastTemplate {
    pub toast_type: ToastType,
    pub message: String,
}

#[derive(Template)]
#[template(path = "views/base/main/admin.html")]
pub struct AdminTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
}

#[derive(Template)]
#[template(path = "views/base/main/employments.html")]
pub struct EmploymentsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub employments: Vec<EmploymentViewModel>,
    pub employment_states: Vec<EmploymentState>,
}

#[derive(Template)]
#[template(path = "views/base/main/employments_table_body.html")]
pub struct EmploymentsTableTemplate {
    pub employments: Vec<EmploymentViewModel>,
}

#[derive(Template)]
#[template(path = "htmx/base/auth/register_success.html")]
pub struct RegisterSuccessTemplate {}

#[derive(Template)]
#[template(path = "partials/form_errors.html")]
pub struct FormErrorsTemplate {
    pub validation_errors: ValidationErrors,
}

#[derive(Template)]
#[template(path = "views/base/main/admin_users.html")]
pub struct AdminUsersTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub users: Vec<UserViewModel>,
}

#[derive(Template)]
#[template(path = "views/base/main/admin_user_details.html")]
pub struct AdminUserDetailsTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
    pub user: User,
}

#[derive(Template)]
#[template(path = "views/base/main/admin_user_create.html")]
pub struct AdminUserCreateTemplate {
    pub session: AuthSession,
    pub active_route: Option<ActiveRoute>,
}

#[derive(Template)]
#[template(path = "views/base/main/admin_users_table.html")]
pub struct AdminUsersTableTemplate {
    pub users: Vec<UserViewModel>,
}
