use eframe::wasm_bindgen::{self, prelude::*};
use log::{debug, error};
use taskify_core::file_dialogue::{open_pdf_file, open_text_file, GenericFileDialogue};
use web_sys::{Request, RequestInit};

fn spawn_async_http_post_for_task_creation_text(data: String) {
    wasm_bindgen_futures::spawn_local(async move {
        debug!("building POST request payload, bytes={}", data.len());
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&wasm_bindgen::JsValue::from_str(&data));

        let runner_url = match std::env::var("RUNNER_URL") {
            Ok(url) => url,
            Err(err) => {
                error!("RUNNER_URL missing: {err}");
                return;
            }
        };

        debug!("{}", format!("{}/task/new_text", runner_url));

        let request =
            Request::new_with_str_and_init(&format!("{}/task/new_text", runner_url), &opts);
        let request = match request {
            Err(err) => {
                error!("failed to create request: {:?}", err);
                return;
            }
            Ok(ok) => {
                debug!("request created");
                ok
            }
        };
        let _ = request.headers().set("Content-Type", "application/json");
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                error!("window is unavailable");
                return;
            }
        };
        let resp_value =
            wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await;
        let _resp_value = match resp_value {
            Err(err) => {
                error!("fetch failed: {:?}", err);
                return;
            }
            Ok(ok) => ok,
        };
    })
}

#[wasm_bindgen]
pub struct WebHandle {
    input_text_dialogue: Option<GenericFileDialogue<Result<String, Box<dyn std::error::Error>>>>,
    input_pdf_dialogue: Option<GenericFileDialogue<Result<String, Box<dyn std::error::Error>>>>,
}

#[wasm_bindgen]
impl WebHandle {
    #[allow(clippy::allow_attributes, clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Redirect `log` macros (debug!/error!/...) to the browser console.
        let log_level = if cfg!(debug_assertions) {
            log::Level::Trace
        } else {
            log::Level::Debug
        };
        console_log::init_with_level(log_level).ok();

        Self {
            input_text_dialogue: None,
            input_pdf_dialogue: None,
        }
    }

    #[wasm_bindgen]
    pub fn scan_text(&mut self) {
        debug!("starting text file dialog");
        self.input_text_dialogue = Some(open_text_file());
    }

    #[wasm_bindgen]
    pub fn scan_pdf(&mut self) {
        debug!("starting pdf file dialog");
        self.input_pdf_dialogue = Some(open_pdf_file());
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        if let Some(mut future) = self.input_text_dialogue.take() {
            if let Some(result) = future.poll() {
                match result {
                    Err(err) => error!("text dialogue error: {err}"),
                    Ok(ok) => {
                        debug!("text dialogue completed, chars={}", ok.len());
                        spawn_async_http_post_for_task_creation_text(ok.clone());
                    }
                }
            } else {
                self.input_text_dialogue = Some(future);
            }
        }

        if let Some(mut future) = self.input_pdf_dialogue.take() {
            if let Some(result) = future.poll() {
                match result {
                    Err(err) => error!("pdf dialogue error: {err}"),
                    Ok(ok) => {
                        debug!("pdf dialogue completed, chars={}", ok.len());
                        spawn_async_http_post_for_task_creation_text(ok.clone());
                    }
                }
            } else {
                self.input_pdf_dialogue = Some(future);
            }
        }
    }

    #[wasm_bindgen]
    pub fn scan_image(&mut self) {
        debug!("scan_image is not implemented yet");
    }
}
