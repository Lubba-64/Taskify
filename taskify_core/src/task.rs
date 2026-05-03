#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Task {
    pub task_id: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub task_start_date: chrono::NaiveDateTime,
    pub task_end_date: chrono::NaiveDateTime,
    pub task_description: String,
    pub task_title: String,
    pub task_priority: String,
}
