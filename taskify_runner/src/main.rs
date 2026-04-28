mod deepseek;
mod schema;
use crate::deepseek::call_deepseek;
use crate::schema::tasks;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug)]
struct RunnerError;

impl Default for RunnerError {
    fn default() -> Self {
        Self {}
    }
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("An error occured with the runner")
    }
}

impl std::error::Error for RunnerError {}

#[derive(Debug)]
struct TaskParseError {
    message: String,
}

impl TaskParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for TaskParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TaskParseError {}

#[derive(serde::Serialize, HasQuery)]
struct Task {
    task_id: i32,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    task_start_date: chrono::NaiveDateTime,
    task_end_date: chrono::NaiveDateTime,
    task_description: String,
    task_title: String,
    task_priority: String,
}
#[derive(serde::Deserialize, Insertable)]
#[diesel(table_name = tasks)]
struct NewTask {
    task_start_date: chrono::NaiveDateTime,
    task_end_date: chrono::NaiveDateTime,
    task_description: String,
    task_title: String,
    task_priority: String,
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    tracing::error!(error = %err, "internal error");
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn build_task_prompt(text: String) -> String {
    return format!(
        "here is some text, do your best and fill out the json with the provided info
    {{
    task_start_date: time,
    task_end_date: time,
    task_description: string,
    task_title: string,
    task_priority: string,
    }}
    {}",
        text
    );
}

fn parse_new_task_from_model_response(raw: &str) -> Result<NewTask, TaskParseError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(TaskParseError::new(
            "model returned empty response".to_string(),
        ));
    }

    if trimmed.eq_ignore_ascii_case("null") {
        return Err(TaskParseError::new(
            "model returned null due to low confidence".to_string(),
        ));
    }

    if let Ok(task) = serde_json::from_str::<NewTask>(trimmed) {
        return Ok(task);
    }

    let start = trimmed.find('{');
    let end = trimmed.rfind('}');
    if let (Some(start), Some(end)) = (start, end) {
        if start < end {
            let candidate = &trimmed[start..=end];
            if let Ok(task) = serde_json::from_str::<NewTask>(candidate) {
                return Ok(task);
            }
        }
    }

    Err(TaskParseError::new(format!(
        "could not parse task JSON from model response: {}",
        raw
    )))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tracing::info!("database url loaded from environment");
    let manager =
        deadpool_diesel::postgres::Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::postgres::Pool::builder(manager)
        .build()
        .unwrap();
    let app = Router::new()
        .route("/task/new", post(new_task))
        .route("/task/new_image", post(new_task_image))
        .route("/task/new_text", post(new_task_text))
        .route("/task/getall", get(get_all_tasks))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!(%addr, "taskify_runner starting");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct NewTaskImage {
    data: Option<String>,
}

#[axum::debug_handler]
async fn new_task_image(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(new_task): Json<NewTaskImage>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let mut text = None::<String>;
    if let Some(image) = new_task.data.map(|x| x.as_bytes().to_vec()) {
        tracing::debug!(bytes = image.len(), "writing OCR input image");
        tokio::fs::write("/tmp/tmpimg", image)
            .await
            .map_err(internal_error)?;
        let mut lt = leptess::LepTess::new(None, "eng").map_err(internal_error)?;
        text = Some(lt.get_utf8_text().map_err(internal_error)?);
        tracing::debug!(
            ocr_chars = text.as_ref().map(|x| x.len()).unwrap_or(0),
            "ocr completed"
        );
        tokio::fs::remove_file("/tmp/tmpimg")
            .await
            .map_err(internal_error)?;
    }
    if text.is_none() {
        tracing::warn!("new_task_image request had no usable image content");
        return Err(internal_error(RunnerError::default()));
    }
    let text = text.expect("impossible to throw");

    let res = match call_deepseek(&build_task_prompt(text)).await {
        Ok(ok) => Ok(ok),
        Err(_err) => Err(internal_error(RunnerError::default())),
    }?;
    let preview: String = res.chars().take(160).collect();
    tracing::debug!(preview = %preview, "raw model response preview");
    let res = parse_new_task_from_model_response(&res).map_err(internal_error)?;
    tracing::debug!("deepseek response parsed into NewTask");

    let conn = pool.get().await.map_err(internal_error)?;
    tracing::debug!("database connection acquired for new_task_image");
    let res = conn
        .interact(|conn| {
            diesel::insert_into(tasks::table)
                .values(res)
                .returning(Task::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

#[derive(serde::Deserialize)]
struct NewTaskText {
    data: Option<String>,
}

#[axum::debug_handler]
async fn new_task_text(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(new_task): Json<NewTaskText>,
) -> Result<Json<Task>, (StatusCode, String)> {
    if new_task.data.is_none() {
        tracing::warn!("new_task_text request had empty payload");
        return Err(internal_error(RunnerError::default()));
    }
    let text = new_task.data.expect("impossible to throw");
    tracing::debug!(chars = text.len(), "received raw text payload");
    let res = match call_deepseek(&build_task_prompt(text)).await {
        Ok(ok) => Ok(ok),
        Err(_err) => Err(internal_error(RunnerError::default())),
    }?;
    let preview: String = res.chars().take(160).collect();
    tracing::debug!(preview = %preview, "raw model response preview");
    let res = parse_new_task_from_model_response(&res).map_err(internal_error)?;
    tracing::debug!("deepseek response parsed into NewTask");

    let conn = pool.get().await.map_err(internal_error)?;
    tracing::debug!("database connection acquired for new_task_text");
    let res = conn
        .interact(|conn| {
            diesel::insert_into(tasks::table)
                .values(res)
                .returning(Task::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

async fn new_task(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(new_task): Json<NewTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    tracing::debug!("database connection acquired for new_task");
    let res = conn
        .interact(|conn| {
            diesel::insert_into(tasks::table)
                .values(new_task)
                .returning(Task::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

async fn get_all_tasks(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<Task>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    tracing::debug!("database connection acquired for get_all_tasks");
    let res = conn
        .interact(|conn| Task::query().load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}
