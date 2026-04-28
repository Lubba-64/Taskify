use crate::file_dialogue::{
    open_image_file, open_pdf_file, open_text_file, post_text, GenericFileDialogue,
};
use log::{debug, error};

pub struct TaskifyApp {
    input_image_dialogue: Option<GenericFileDialogue<Result<Vec<u8>, Box<dyn std::error::Error>>>>,
    input_text_dialogue: Option<GenericFileDialogue<Result<String, Box<dyn std::error::Error>>>>,
    input_pdf_dialogue: Option<GenericFileDialogue<Result<String, Box<dyn std::error::Error>>>>,
    err_str: String,
}

impl TaskifyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input_image_dialogue: None,
            input_pdf_dialogue: None,
            input_text_dialogue: None,
            err_str: "".to_string(),
        }
    }
}

// TODO: ADD ERROR UI
impl eframe::App for TaskifyApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if frame.is_web() {
                ui.label("We are rendering from WASM");
                ui.separator();
            }
            ui.label("outside of wasm text");
            ui.label(self.err_str.clone());

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
            let mut clear = false;
            if let Some(ref mut future) = &mut self.input_text_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => error!("text dialogue error: {err}"),
                        Ok(ok) => {
                            debug!("text dialogue completed, chars={}", ok.len());
                            match post_text(ok.clone()) {
                                Err(err) => self.err_str = format!("{}", err),
                                Ok(_ok) => {}
                            }
                            clear = true;
                        }
                    },
                    None => {}
                }
            }
            if clear {
                self.input_text_dialogue = None;
            }
            if ui.button("Scan PDF For Task").clicked() {
                self.input_pdf_dialogue = Some(open_pdf_file());
            }
            let mut clear = false;
            if let Some(ref mut future) = &mut self.input_pdf_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => error!("pdf dialogue error: {err}"),
                        Ok(ok) => {
                            debug!("pdf dialogue completed, chars={}", ok.len());
                            match post_text(ok.clone()) {
                                Err(err) => self.err_str = format!("{}", err),
                                Ok(_ok) => {}
                            }
                            clear = true;
                        }
                    },
                    None => {}
                }
            }
            if clear {
                self.input_pdf_dialogue = None;
            }
        });
    }
}
