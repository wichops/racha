use askama::Template;
use askama_web::WebTemplate;
use crate::models::group::{Group, MemberWithStreaks};

#[derive(Template, WebTemplate)]
#[template(path = "groups/feed.html")]
pub struct GroupFeedTemplate {
    pub group: Group,
    pub members_grouped: Vec<(String, Vec<MemberWithStreaks>)>,
    pub flash_message: Option<String>,
    pub flash_is_error: bool,
}

#[derive(Template, WebTemplate)]
#[template(path = "groups/_create_form.html")]
pub struct CreateGroupFormPartial;

#[derive(Template, WebTemplate)]
#[template(path = "groups/_join_form.html")]
pub struct JoinGroupFormPartial;
