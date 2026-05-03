use crate::{
    file_dialogue_component::{FileDialogError, FileDialoge},
    post_file::{post_file, PostFileError},
};
use image::ImageReader;
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

type FilePromise = Arc<Mutex<Option<Result<(), Box<PostFileError>>>>>;

pub struct TaskifyApp {
    err_str: String,
    pdf_err: FilePromise,
    txt_err: FilePromise,
    img_err: FilePromise,
}

impl TaskifyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            err_str: "".to_string(),
            pdf_err: Arc::new(Mutex::new(None)),
            txt_err: Arc::new(Mutex::new(None)),
            img_err: Arc::new(Mutex::new(None)),
        }
    }
}

fn check_file_is_image(file: Vec<u8>) -> Result<String, Box<FileDialogError>> {
    let _ = ImageReader::new(Cursor::new(file.clone()))
        .with_guessed_format()
        .map_err(|e| FileDialogError::new(format!("{:#?}", e)))?
        .decode()
        .map_err(|e| FileDialogError::new(format!("{:#?}", e)))?;
    Ok(String::from_utf8(file).map_err(|e| FileDialogError::new(format!("{:#?}", e)))?)
}

fn try_to_utf8(file: Vec<u8>) -> Result<String, Box<FileDialogError>> {
    Ok(String::from_utf8(file).map_err(|e| FileDialogError::new(format!("{:#?}", e)))?)
}

fn extract_pdf_text(file: Vec<u8>) -> Result<String, Box<FileDialogError>> {
    Ok(pdf_extract::extract_text_from_mem(&file)
        .map_err(|e| FileDialogError::new(format!("{:#?}", e)))?)
}

impl eframe::App for TaskifyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(&self.err_str);
            let pdf_promise_clone = self.pdf_err.clone();
            ui.add(&mut FileDialoge::new(
                &["pdf"],
                Box::new(extract_pdf_text),
                Some(Box::new(move |data| {
                    let _ = pdf_promise_clone
                        .lock()
                        .expect("expect lock")
                        .insert(post_file(data, "task/new_text".to_string()));
                    Ok(())
                })),
                "Create Task From PDF",
            ));

            let txt_promise_clone = self.txt_err.clone();
            ui.add(&mut FileDialoge::new(
                &["txt", "md"],
                Box::new(try_to_utf8),
                Some(Box::new(move |data| {
                    let _ = txt_promise_clone
                        .lock()
                        .expect("expect lock")
                        .insert(post_file(data, "task/new_text".to_string()));
                    Ok(())
                })),
                "Create Task From Text",
            ));
            let img_promise_clone = self.img_err.clone();
            ui.add(&mut FileDialoge::new(
                &["png", "jpg", "jpeg"],
                Box::new(check_file_is_image),
                Some(Box::new(move |data| {
                    let _ = img_promise_clone
                        .lock()
                        .expect("expect lock")
                        .insert(post_file(data, "task/new_image".to_string()));
                    Ok(())
                })),
                "Create Task From Image",
            ));
            if let Some(result) = self.pdf_err.lock().expect("expect lock").take() {
                match result {
                    Ok(_) => {}
                    Err(e) => self.err_str = format!("{:#?}", e),
                }
            }
            if let Some(result) = self.txt_err.lock().expect("expect lock").take() {
                match result {
                    Ok(_) => {}
                    Err(e) => self.err_str = format!("{:#?}", e),
                }
            }
            if let Some(result) = self.img_err.lock().expect("expect lock").take() {
                match result {
                    Ok(_) => {}
                    Err(e) => self.err_str = format!("{:#?}", e),
                }
            }
        });
    }
}
