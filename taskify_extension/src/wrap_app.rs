use taskify_core::file_dialogue::{open_image_file, GenericFileDialogue};

pub struct TaskifyExtensionApp {
    input_image_dialogue: Option<GenericFileDialogue<Result<Vec<u8>, Box<dyn std::error::Error>>>>,
    input_text_dialogue: Option<GenericFileDialogue<Result<Vec<u8>, Box<dyn std::error::Error>>>>,
    input_pdf_dialogue: Option<GenericFileDialogue<Result<Vec<u8>, Box<dyn std::error::Error>>>>,
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
                        Err(err) => {
                            // TODO: ADD ERROR UI
                        }
                        Ok(ok) => {
                            // TODO: send to runner/database
                        }
                    },
                    None => {}
                }
            }
            if ui.button("Scan Text For Task").clicked() {
                self.input_text_dialogue = Some(open_image_file());
            }
            if let Some(ref mut future) = &mut self.input_text_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => {
                            // TODO: ADD ERROR UI
                        }
                        Ok(ok) => {
                            // TODO: send to runner/database
                        }
                    },
                    None => {}
                }
            }
            if ui.button("Scan PDF For Task").clicked() {
                self.input_pdf_dialogue = Some(open_image_file());
            }
            if let Some(ref mut future) = &mut self.input_pdf_dialogue {
                match future.poll() {
                    Some(result) => match result {
                        Err(err) => {
                            // TODO: ADD ERROR UI
                        }
                        Ok(ok) => {
                            // TODO: send to runner/database
                        }
                    },
                    None => {}
                }
            }
        });
    }
}
