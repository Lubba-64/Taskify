// This file is a good example of webassembly making things much more complicated.
#[cfg(target_arch = "wasm32")]
use futures::Future;
#[cfg(target_arch = "wasm32")]
use futures::FutureExt;
use image::ImageReader;
use log::{debug, error};
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
#[cfg(target_arch = "wasm32")]
use rfd::{AsyncFileDialog, FileHandle};
#[cfg(target_arch = "wasm32")]
use std::cell::Cell;
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use std::panic;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use wasm_bindgen;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures;
#[cfg(target_arch = "wasm32")]
use web_sys::{Request, RequestInit};

pub struct GenericFileDialogue<T: 'static> {
    obj: Option<T>,
    #[cfg(target_arch = "wasm32")]
    wasm_task: FileDialogueWasmTask<T>,
}

unsafe impl<T> std::marker::Sync for GenericFileDialogue<T> {}

impl<T: 'static> GenericFileDialogue<T> {
    #[cfg(not(target_arch = "wasm32"))]
    fn new(obj: T) -> GenericFileDialogue<T> {
        return Self { obj: Some(obj) };
    }

    #[cfg(target_arch = "wasm32")]
    fn new(task: FileDialogueWasmTask<T>) -> GenericFileDialogue<T> {
        return Self {
            obj: None::<T>,
            wasm_task: task,
        };
    }

    pub fn poll(&mut self) -> &Option<T> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(output) = self.wasm_task.take_output() {
                self.obj = Some(output.expect("file dialog async task panicked"));
            }
            &self.obj
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            &self.obj
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct FileDialogueWasmTask<T>(Rc<Cell<Option<thread::Result<T>>>>);

#[cfg(target_arch = "wasm32")]
impl<T: 'static> FileDialogueWasmTask<T> {
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

#[derive(Debug)]
struct FileDialogueError;

impl Default for FileDialogueError {
    fn default() -> Self {
        Self {}
    }
}

impl std::fmt::Display for FileDialogueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("An error occured with the file dialogue module")
    }
}

impl std::error::Error for FileDialogueError {}

#[cfg(target_arch = "wasm32")]
async fn get_bytes(file_handle: Option<FileHandle>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let fh = match file_handle {
        None => {
            return Err(Box::new(FileDialogueError::default()));
        }
        Some(x) => x,
    };
    Ok(fh.read().await)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_bytes(path_buf: Option<PathBuf>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path_buf = match path_buf {
        None => {
            return Err(Box::new(FileDialogueError::default()));
        }
        Some(x) => x,
    };
    let path = match path_buf.into_os_string().into_string() {
        Ok(x) => x,
        Err(_) => return Err(Box::new(FileDialogueError::default())),
    };
    Ok(path.as_bytes().to_vec())
}

macro_rules! make_generic_dialogue {
    ($sync_name:ident, $async_name:ident, $generic_name:ident, $extensions:expr, $convert:ident, $ret: ty) => {
        #[cfg(not(target_arch = "wasm32"))]
        fn $sync_name() -> Result<$ret, Box<dyn std::error::Error>> {
            let file = get_bytes(
                FileDialog::new()
                    .add_filter("text", $extensions)
                    .set_directory("./")
                    .pick_file(),
            )?;
            Ok($convert(file)?)
        }
        #[cfg(target_arch = "wasm32")]
        async fn $async_name() -> Result<$ret, Box<dyn std::error::Error>> {
            let file_handle = AsyncFileDialog::new()
                .add_filter("text", $extensions)
                .pick_file()
                .await;
            let file = match get_bytes(file_handle).await {
                Ok(file) => file,
                Err(err) => {
                    return Err(err);
                }
            };
            Ok($convert(file)?)
        }
        pub fn $generic_name() -> GenericFileDialogue<Result<$ret, Box<dyn std::error::Error>>> {
            #[cfg(target_arch = "wasm32")]
            {
                GenericFileDialogue::new(FileDialogueWasmTask::spawn($async_name()))
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                GenericFileDialogue::new($sync_name())
            }
        }
    };
}

fn check_file_is_image(file: Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let _ = ImageReader::new(Cursor::new(file.clone()))
        .with_guessed_format()?
        .decode()?; //we do not need to use this value, we just need to confirm its an image so we can pass it along to the database.
    Ok(file)
}

make_generic_dialogue!(
    open_image_file_sync,
    open_image_file_async,
    open_image_file,
    &["png", "jpg", "jpeg"],
    check_file_is_image,
    Vec<u8>
);

fn try_to_utf8(file: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::from_utf8(file)?)
}

make_generic_dialogue!(
    open_text_file_sync,
    open_text_file_async,
    open_text_file,
    &["txt", "md"],
    try_to_utf8,
    String
);

fn extract_pdf_text(file: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
    return Ok(pdf_extract::extract_text_from_mem(&file)?);
}

make_generic_dialogue!(
    open_pdf_file_sync,
    open_pdf_file_async,
    open_pdf_file,
    &["pdf"],
    extract_pdf_text,
    String
);

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
        .post(&format!("{}/task/new_text", runner_url))
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
