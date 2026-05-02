use std::io::Cursor;

use image::ImageReader;
use log::{debug, error};

use crate::file_dialogue_component::{FileDialogError, FileDialoge};

#[cfg(target_arch = "wasm32")]
fn post_text_wasm(data: String) -> Result<(), Box<dyn std::error::Error>> {
    wasm_bindgen_futures::spawn_local(async move {
        debug!("building POST request payload, bytes={}", data.len());
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&wasm_bindgen::JsValue::from_str(&format!(
            "{{data:{}}}",
            &data
        )));
        let runner_url = match std::env::var("RUNNER_URL") {
            Ok(url) => url,
            Err(err) => {
                error!("RUNNER_URL missing: {err}");
                return;
            }
        };
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

#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Serialize)]
struct NewTaskText {
    data: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
fn post_text_desktop(data: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    let runner_url = std::env::var("RUNNER_URL")?;
    let _response = client
        .post(format!("{}/task/new_text", runner_url))
        .json(&NewTaskText {
            data: Some(String::from_utf8(std::fs::read(data)?)?),
        })
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    Ok(())
}

pub fn post_text(data: String) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        post_text_desktop(data)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        post_text_wasm(data);
    }
    Ok(())
}

pub struct TaskifyApp {
    err_str: String,
}

impl TaskifyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            err_str: "".to_string(),
        }
    }
}

fn check_file_is_image(file: Vec<u8>) -> Result<Vec<u8>, Box<FileDialogError>> {
    let _ = ImageReader::new(Cursor::new(file.clone()))
        .with_guessed_format()
        .map_err(|e| FileDialogError::new(format!("{:#?}", e)))?
        .decode()
        .map_err(|e| FileDialogError::new(format!("{:#?}", e)))?; //we do not need to use this value, we just need to confirm its an image so we can pass it along to the database.
    Ok(file)
}

fn try_to_utf8(file: Vec<u8>) -> Result<String, Box<FileDialogError>> {
    Ok(String::from_utf8(file).map_err(|e| FileDialogError::new(format!("{:#?}", e)))?)
}

fn extract_pdf_text(file: Vec<u8>) -> Result<String, Box<FileDialogError>> {
    Ok(pdf_extract::extract_text_from_mem(&file)
        .map_err(|e| FileDialogError::new(format!("{:#?}", e)))?)
}

// TODO: ADD ERROR UI
impl eframe::App for TaskifyApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(&mut FileDialoge::new(
                &["pdf"],
                Box::new(extract_pdf_text),
                Some(Box::new(|_| {})),
                "Scan Task From PDF",
            ))
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(&mut FileDialoge::new(
                &["txt", "md"],
                Box::new(extract_pdf_text),
                Some(Box::new(|_| {})),
                "Scan Task From Text",
            ))
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add(&mut FileDialoge::new(
                &["pdf"],
                Box::new(extract_pdf_text),
                Some(Box::new(|_| {})),
                "Open File",
            ))
        });
    }
}
