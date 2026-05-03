use crate::{
    file_dialogue_component::{FileDialogError, FileDialoge},
    post_file::{post_file, PostFileError},
};
use egui::containers::Frame;
use image::ImageReader;
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

type FilePromise = Arc<Mutex<Option<Result<(), Box<PostFileError>>>>>;

#[derive(serde::Serialize, serde::Deserialize)]
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

pub struct TaskifyApp {
    err_str: String,
    pdf_err: FilePromise,
    txt_err: FilePromise,
    img_err: FilePromise,
    tasks: Vec<Task>,
}

impl TaskifyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            err_str: "".to_string(),
            pdf_err: Arc::new(Mutex::new(None)),
            txt_err: Arc::new(Mutex::new(None)),
            img_err: Arc::new(Mutex::new(None)),
            tasks: get_tasks_temp_desktop().unwrap_or_default(),
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

fn get_tasks_temp_desktop() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    let runner_url =
        std::env::var("RUNNER_URL").map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    let response = client
        .get(format!("{}/{}", runner_url, "task/getall"))
        .send()
        .map_err(|e| PostFileError::new(format!("{:#?}", e)))?;
    Ok(response.json::<Vec<Task>>()?)
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

            if ui.button("Refresh Tasks").clicked() {
                self.tasks = get_tasks_temp_desktop().unwrap_or_default();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for task in &self.tasks {
                    let frame = Frame::group(ui.style())
                        .fill(ui.visuals().panel_fill)
                        .inner_margin(egui::Margin::same(10));
                    frame.show(ui, |ui| {
                        ui.label(&task.task_title);
                        ui.label(&task.task_description);
                        ui.label(&task.task_priority);
                        ui.label(format!("start date: {:#?}", task.task_start_date));
                        ui.label(format!("end date: {:#?}", task.task_end_date));
                    });
                }
            });
        });
    }
}
