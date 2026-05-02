// This file is a good example of webassembly making things much more complicated.
// However, the resulting api is beautiful.
use egui::Response;
use egui::Ui;
use egui::Widget;
#[cfg(target_arch = "wasm32")]
use futures::Future;
#[cfg(target_arch = "wasm32")]
use futures::FutureExt;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(target_arch = "wasm32")]
use rfd::{AsyncFileDialog, FileHandle};
#[cfg(target_arch = "wasm32")]
use std::cell::Cell;
#[cfg(target_arch = "wasm32")]
use std::panic;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::thread;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit};

pub struct GenericFileDialog<T: Clone + 'static> {
    obj: Option<T>,
    #[cfg(target_arch = "wasm32")]
    wasm_task: FileDialogWasmTask<T>,
}

unsafe impl<T: Clone + 'static> std::marker::Sync for GenericFileDialog<T> {}

impl<T: Clone + 'static> GenericFileDialog<T> {
    #[cfg(not(target_arch = "wasm32"))]
    fn new(obj: T) -> GenericFileDialog<T> {
        Self { obj: Some(obj) }
    }

    #[cfg(target_arch = "wasm32")]
    fn new(task: FileDialogWasmTask<T>) -> GenericFileDialog<T> {
        return Self {
            obj: None::<T>,
            wasm_task: task,
        };
    }

    pub fn poll(&mut self) -> Option<T> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(output) = self.wasm_task.take_output() {
                self.obj = Some(output.expect("file dialog async task panicked"));
            }
            self.obj
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.obj.clone()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct FileDialogWasmTask<T>(Rc<Cell<Option<thread::Result<T>>>>);

#[cfg(target_arch = "wasm32")]
impl<T: 'static> FileDialogWasmTask<T> {
    pub fn spawn<F: 'static + Future<Output = T>>(future: F) -> Self {
        let sender = Rc::new(Cell::new(None));
        let receiver = sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let future = panic::AssertUnwindSafe(future).catch_unwind();
            sender.set(Some(future.await));
        });
        Self(receiver)
    }
    pub fn take_output(&self) -> Option<thread::Result<T>> {
        self.0.take()
    }
}

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
    extensions: &[impl ToString],
) -> Result<T, Box<FileDialogError>> {
    let file_handle = AsyncFileDialog::new()
        .add_filter("text", extensions)
        .pick_file()
        .await;
    let file = match get_bytes(file_handle).await {
        Ok(file) => file,
        Err(err) => {
            return Err(err);
        }
    };
    Ok(convert(file)?)
}

fn file_dialogue<T: Clone + 'static, F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone>(
    convert: Box<F>,
    extensions: Vec<String>,
) -> GenericFileDialog<Result<T, Box<FileDialogError>>> {
    #[cfg(target_arch = "wasm32")]
    {
        GenericFileDialog::new(FileDialogWasmTask::spawn(async_file_dialogue(
            convert,
            extensions.as_slice(),
        )))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        GenericFileDialog::new(
            sync_file_dialogue(convert, extensions.as_slice())
                .map_err(|e| Box::new(FileDialogError::new(format!("{:#?}", e)))),
        )
    }
}

#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct FileDialoge<
    T: Clone + 'static,
    F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone,
> {
    generic_file_dialogue: Option<GenericFileDialog<Result<T, Box<FileDialogError>>>>,
    err: Option<Box<dyn std::error::Error>>,
    extensions: Vec<String>,
    convert: Box<F>,
    handler: Option<Box<dyn Fn(T)>>,
    label: String,
}

impl<T: Clone + 'static, F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone>
    FileDialoge<T, F>
{
    pub fn new(
        extensions: &[&str],
        convert: Box<F>,
        handler: Option<Box<dyn Fn(T)>>,
        labal: &str,
    ) -> Self {
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

impl<T: Clone + 'static, F: Fn(Vec<u8>) -> Result<T, Box<FileDialogError>> + Clone> Widget
    for &mut FileDialoge<T, F>
{
    fn ui(self, ui: &mut Ui) -> Response {
        if ui.button(self.label.clone()).clicked() {
            self.generic_file_dialogue =
                Some(file_dialogue(self.convert.clone(), self.extensions.clone()));
        }
        ui.colored_label(egui::Color32::RED, format!("{:#?}", self.err));
        let mut clear = false;
        if let Some(ref mut future) = &mut self.generic_file_dialogue {
            if let Some(result) = future.poll() {
                let result = result.clone();
                match result {
                    Err(err) => self.err = Some(err),
                    Ok(ok) => {
                        if let Some(handler) = &self.handler {
                            handler(ok);
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
