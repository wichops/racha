use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
    pub flash_message: Option<String>,
    pub flash_is_error: bool,
}

impl LoginTemplate {
    pub fn with_error(msg: &str) -> Self {
        Self {
            error: Some(msg.to_string()),
            flash_message: None,
            flash_is_error: false,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "auth/register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
    pub flash_message: Option<String>,
    pub flash_is_error: bool,
}

impl RegisterTemplate {
    pub fn with_error(msg: &str) -> Self {
        Self {
            error: Some(msg.to_string()),
            flash_message: None,
            flash_is_error: false,
        }
    }
}
