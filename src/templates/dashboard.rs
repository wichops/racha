use askama::Template;
use askama_web::WebTemplate;
use crate::models::task::TaskWithStreak;
use crate::models::group::GroupWithMembership;

#[derive(Template, WebTemplate)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub username: String,
    pub tasks: Vec<TaskWithStreak>,
    pub groups: Vec<GroupWithMembership>,
    pub flash_message: Option<String>,
    pub flash_is_error: bool,
}
