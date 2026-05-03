use crate::{
    file_dialogue_component::{FileDialogError, FileDialoge},
    get_tasks::{get_tasks, GetTasksError},
    post_file::{post_file, PostFileError},
    task::Task,
    wasm_task::TaskHandler,
};
use egui::containers::Frame;
use image::ImageReader;
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

type FilePromise = Arc<Mutex<Option<TaskHandler<Result<(), Box<PostFileError>>>>>>;
type GetTasksPromise = Option<TaskHandler<Result<Vec<Task>, Box<GetTasksError>>>>;

pub struct TaskifyApp {
    err_str: String,
    pdf_err: FilePromise,
    txt_err: FilePromise,
    img_err: FilePromise,
    get_tasks: GetTasksPromise,
    tasks: Vec<Task>,
    init: bool,
}

impl TaskifyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            err_str: "".to_string(),
            pdf_err: Arc::new(Mutex::new(None)),
            txt_err: Arc::new(Mutex::new(None)),
            img_err: Arc::new(Mutex::new(None)),
            get_tasks: None,
            tasks: vec![],
            init: false,
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
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title("Taskify".to_string()));
            ui.heading("Taskify");
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
            if let Some(mut promise) = self.pdf_err.lock().expect("expect lock").take() {
                if let Some(result) = promise.poll() {
                    let _ = result.inspect_err(|e| {
                        self.err_str = format!("{:#?}", e);
                    });
                }
            }
            if let Some(mut promise) = self.txt_err.lock().expect("expect lock").take() {
                if let Some(result) = promise.poll() {
                    let _ = result.inspect_err(|e| {
                        self.err_str = format!("{:#?}", e);
                    });
                }
            }
            if let Some(mut promise) = self.img_err.lock().expect("expect lock").take() {
                if let Some(result) = promise.poll() {
                    let _ = result.inspect_err(|e| {
                        self.err_str = format!("{:#?}", e);
                    });
                }
            }
            if ui.button("Refresh Tasks").clicked() || !self.init {
                self.get_tasks = Some(get_tasks());
                self.init = true;
            }
            if let Some(mut promise) = self.get_tasks.take() {
                if let Some(result) = promise.poll() {
                    let _ = result
                        .inspect_err(|e| {
                            self.err_str = format!("{:#?}", e);
                        })
                        .inspect(|tasks| {
                            self.tasks = tasks.clone();
                        });
                }
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
