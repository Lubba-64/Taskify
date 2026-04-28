use log::{debug, error, info};
use taskify_core::file_dialogue::{
    open_image_file, open_pdf_file, open_text_file, GenericFileDialogue,
};
use web_sys::{Request, RequestInit};

pub struct TaskifyExtensionApp {
    input_image_dialogue: Option<GenericFileDialogue<Result<Vec<u8>, Box<dyn std::error::Error>>>>,
    input_text_dialogue: Option<GenericFileDialogue<Result<String, Box<dyn std::error::Error>>>>,
    input_pdf_dialogue: Option<GenericFileDialogue<Result<String, Box<dyn std::error::Error>>>>,
}

impl TaskifyExtensionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input_image_dialogue: None,
            input_pdf_dialogue: None,
            input_text_dialogue: None,
        }
    }
}

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

// TODO: ADD ERROR UI
impl eframe::App for TaskifyExtensionApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if frame.is_web() {
                ui.label("We are rendering from WASM");
                ui.separator();
            }
            ui.label("outside of wasm text");

            if ui.button("Scan Image For Task").clicked() {
                self.input_image_dialogue = Some(open_image_file());
            }
            if let Some(ref mut future) = &mut self.input_image_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => error!("image dialogue error: {err}"),
                        Ok(ok) => debug!("image dialogue completed, bytes={}", ok.len()),
                    },
                    None => {}
                }
            }
            if ui.button("Scan Text For Task").clicked() {
                self.input_text_dialogue = Some(open_text_file());
            }
            if let Some(ref mut future) = &mut self.input_text_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => error!("text dialogue error: {err}"),
                        Ok(ok) => {
                            debug!("text dialogue completed, chars={}", ok.len());
                            spawn_async_http_post_for_task_creation_text(ok.clone())
                        }
                    },
                    None => {}
                }
            }
            if ui.button("Scan PDF For Task").clicked() {
                self.input_pdf_dialogue = Some(open_pdf_file());
            }
            if let Some(ref mut future) = &mut self.input_pdf_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => error!("pdf dialogue error: {err}"),
                        Ok(ok) => {
                            debug!("pdf dialogue completed, chars={}", ok.len());
                            spawn_async_http_post_for_task_creation_text(ok.clone())
                        }
                    },
                    None => {}
                }
            }
        });
    }
}
