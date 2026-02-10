use askama::Template;
use askama_web::WebTemplate;
use crate::models::task::TaskWithStreak;

#[derive(Template, WebTemplate)]
#[template(path = "tasks/_task_card.html")]
pub struct TaskCardPartial {
    pub task: TaskWithStreak,
}

#[derive(Template, WebTemplate)]
#[template(path = "tasks/_task_form.html")]
pub struct TaskFormPartial;

#[derive(Template, WebTemplate)]
#[template(path = "tasks/_task_edit.html")]
pub struct TaskEditPartial {
    pub task: TaskWithStreak,
}
