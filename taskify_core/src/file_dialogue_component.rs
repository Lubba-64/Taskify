// This file is a good example of webassembly making things much more complicated.
// However, the resulting api is beautiful.
use egui::Response;
use egui::Ui;
use egui::Widget;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(target_arch = "wasm32")]
use rfd::{AsyncFileDialog, FileHandle};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::wasm_task::TaskHandler;

#[derive(Debug, Clone)]
pub struct FileDialogError {
    message: String,
}

impl From<Box<dyn std::error::Error>> for Box<FileDialogError> {
    fn from(value: Box<dyn std::error::Error>) -> Self {
        Box::new(FileDialogError::new(format!("{:#?}", value)))
    }
}

impl FileDialogError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for FileDialogError {
    fn default() -> Self {
        Self::new("An error occurred with the file dialogue module".to_string())
    }
}

impl std::fmt::Display for FileDialogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FileDialogError {}

#[cfg(target_arch = "wasm32")]
async fn get_bytes(file_handle: Option<FileHandle>) -> Result<Vec<u8>, Box<FileDialogError>> {
    let fh = match file_handle {
        None => {
            return Err(Box::new(FileDialogError::default()));
        }
        Some(x) => x,
    };
    Ok(fh.read().await)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_bytes(path_buf: Option<PathBuf>) -> Result<Vec<u8>, Box<FileDialogError>> {
    let path_buf = path_buf.ok_or(Box::new(FileDialogError::default()))?;
    let path = path_buf
        .into_os_string()
        .into_string()
        .map_err(|e| Box::new(FileDialogError::new(format!("{:#?}", e))))?;
    Ok(path.as_bytes().to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_file_dialogue<T>(
    convert: impl FnOnce(Vec<u8>) -> Result<T, Box<FileDialogError>>,
    extensions: &[impl ToString],
) -> Result<T, Box<FileDialogError>> {
    let file = get_bytes(
        FileDialog::new()
            .add_filter("text", extensions)
            .set_directory("./")
            .pick_file(),
    )?;
    convert(file)
}

#[cfg(target_arch = "wasm32")]
async fn async_file_dialogue<T>(
    convert: impl FnOnce(Vec<u8>) -> Result<T, Box<FileDialogError>>,
    extensions: Vec<impl ToString>,
) -> Result<T, Box<FileDialogError>> {
    let file_handle = AsyncFileDialog::new()
        .add_filter("text", extensions.as_slice())
        .pick_file()
        .await;
    let file = match get_bytes(file_handle).await {
        Ok(file) => file,
        Err(err) => {
            return Err(err);
        }
    };
    convert(file)
}

fn file_dialogue<
    T: Clone + 'static,
    F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone + 'static,
>(
    convert: Box<F>,
    extensions: Vec<String>,
) -> TaskHandler<Result<T, Box<FileDialogError>>> {
    #[cfg(target_arch = "wasm32")]
    {
        TaskHandler::new(async_file_dialogue(convert, extensions))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        TaskHandler::new(
            sync_file_dialogue(convert, extensions.as_slice())
                .map_err(|e| Box::new(FileDialogError::new(format!("{:#?}", e)))),
        )
    }
}

type Handler<T> = Option<Box<dyn Fn(T) -> Result<(), Box<FileDialogError>>>>;

#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct FileDialoge<
    T: Clone + 'static,
    F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone,
> {
    generic_file_dialogue: Option<TaskHandler<Result<T, Box<FileDialogError>>>>,
    err: Option<Box<dyn std::error::Error>>,
    extensions: Vec<String>,
    convert: Box<F>,
    handler: Handler<T>,
    label: String,
}

impl<T: Clone + 'static, F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone>
    FileDialoge<T, F>
{
    pub fn new(extensions: &[&str], convert: Box<F>, handler: Handler<T>, labal: &str) -> Self {
        Self {
            generic_file_dialogue: None,
            err: None,
            extensions: extensions.iter().map(|x| x.to_string()).collect(),
            convert,
            label: labal.to_string(),
            handler,
        }
    }
}

impl<T: Clone + 'static, F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone + 'static> Widget
    for &mut FileDialoge<T, F>
{
    fn ui(self, ui: &mut Ui) -> Response {
        if ui.button(self.label.clone()).clicked() {
            self.generic_file_dialogue =
                Some(file_dialogue(self.convert.clone(), self.extensions.clone()));
        }
        if self.err.is_some() {
            ui.colored_label(egui::Color32::RED, format!("{:#?}", self.err));
        }
        let mut clear = false;
        if let Some(ref mut future) = &mut self.generic_file_dialogue {
            if let Some(result) = future.poll() {
                let result = result.clone();
                match result {
                    Err(err) => self.err = Some(err),
                    Ok(ok) => {
                        if let Some(handler) = &self.handler {
                            match handler(ok) {
                                Ok(_) => {}
                                Err(err) => self.err = Some(err),
                            }
                        }
                        clear = true;
                    }
                }
            }
        }
        if clear {
            self.generic_file_dialogue = None;
        }
        ui.response()
    }
}
