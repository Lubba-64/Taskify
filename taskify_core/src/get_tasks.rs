use crate::task::Task;
use crate::wasm_task::TaskHandler;

#[derive(Debug, Clone)]
pub struct GetTasksError {
    message: String,
}

impl From<Box<dyn std::error::Error>> for Box<GetTasksError> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Box::new(GetTasksError::new(format!("{:#?}", value)))
    }
}

impl GetTasksError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for GetTasksError {
    fn default() -> Self {
        Self::new("An error occurred with posting a file".to_string())
    }
}

impl std::fmt::Display for GetTasksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for GetTasksError {}

#[cfg(target_arch = "wasm32")]
async fn get_tasks_async() -> Result<Vec<Task>, Box<GetTasksError>> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| GetTasksError::new(format!("{:#?}", e)))?;
    let runner_url =
        std::env::var("RUNNER_URL").map_err(|e| GetTasksError::new(format!("{:#?}", e)))?;
    let response = client
        .get(format!("{}/{}", runner_url, "task/getall"))
        .send()
        .await
        .map_err(|e| GetTasksError::new(format!("{:#?}", e)))?;
    Ok(response
        .json::<Vec<Task>>()
        .await
        .map_err(|e| GetTasksError::new(format!("{:#?}", e)))?)
}

#[cfg(not(target_arch = "wasm32"))]
fn get_tasks_sync() -> Result<Vec<Task>, Box<GetTasksError>> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| GetTasksError::new(format!("{:#?}", e)))?;
    let runner_url =
        std::env::var("RUNNER_URL").map_err(|e| GetTasksError::new(format!("{:#?}", e)))?;
    let response = client
        .get(format!("{}/{}", runner_url, "task/getall"))
        .send()
        .map_err(|e| GetTasksError::new(format!("{:#?}", e)))?;
    Ok(response
        .json::<Vec<Task>>()
        .map_err(|e| GetTasksError::new(format!("{:#?}", e)))?)
}

pub fn get_tasks() -> TaskHandler<Result<Vec<Task>, Box<GetTasksError>>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        TaskHandler::new(get_tasks_sync())
    }
    #[cfg(target_arch = "wasm32")]
    {
        TaskHandler::new(get_tasks_async())
    }
}
