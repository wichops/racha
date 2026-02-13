use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "profile.html")]
pub struct ProfileTemplate {
    pub username: String,
    pub email: String,
    pub flash_message: Option<String>,
    pub flash_is_error: bool,
}
