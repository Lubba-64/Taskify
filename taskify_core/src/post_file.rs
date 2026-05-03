#[derive(Debug, Clone)]
pub struct PostFileError {
    message: String,
}

impl From<Box<dyn std::error::Error>> for Box<PostFileError> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Box::new(PostFileError::new(format!("{:#?}", value)))
    }
}

impl PostFileError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for PostFileError {
    fn default() -> Self {
        Self::new("An error occurred with posting a file".to_string())
    }
}

impl std::fmt::Display for PostFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PostFileError {}

#[derive(serde::Serialize)]
struct NewTaskText {
    data: Option<String>,
}

#[cfg(target_arch = "wasm32")]
async fn post_file_async(data: String, route: String) -> Result<(), Box<PostFileError>> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    let runner_url =
        std::env::var("RUNNER_URL").map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    let _response = client
        .post(format!("{}/{}", runner_url, route))
        .json(&NewTaskText {
            data: Some(
                String::from_utf8(
                    std::fs::read(data).map_err(|e| PostFileError::new(format!("{:#?}", e)))?,
                )
                .map_err(|e| PostFileError::new(format!("{:#?}", e)))?,
            ),
        })
        .send()
        .await
        .map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn post_file_sync(data: String, route: String) -> Result<(), Box<PostFileError>> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    let runner_url =
        std::env::var("RUNNER_URL").map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    let _response = client
        .post(format!("{}/{}", runner_url, route))
        .json(&NewTaskText {
            data: Some(
                String::from_utf8(
                    std::fs::read(data).map_err(|e| PostFileError::new(format!("{:#?}", e)))?,
                )
                .map_err(|e| PostFileError::new(format!("{:#?}", e)))?,
            ),
        })
        .send()
        .map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    Ok(())
}

pub fn post_file(data: String, route: String) -> Result<(), Box<PostFileError>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        post_file_sync(data, route)
    }
    #[cfg(target_arch = "wasm32")]
    {
        // no error handling because I cannot be bothered. sorry.
        // you will know if it doesnt work but it will fail silently
        wasm_bindgen_futures::spawn_local(async move {
            let _ = post_file_async(data, route).await;
        });
        Ok(())
    }
}
