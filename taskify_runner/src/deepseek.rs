use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub message: ChatMessage,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

pub async fn call_deepseek(prompt: &str) -> Result<String, String> {
    tracing::debug!("Calling DeepSeek API for text generation");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let request = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        temperature: 0.7,
    };

    let api_key = std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY not set");

    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let data: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let result = data
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| "No response from DeepSeek".to_string());

    if result.is_ok() {
        tracing::debug!("DeepSeek API call successful");
    } else {
        tracing::error!("DeepSeek API call failed");
    }

    result
}
