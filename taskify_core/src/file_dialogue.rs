// This file is a good example of webassembly making things much more complicated.
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

use image::ImageReader;
use std::io::Cursor;

pub struct GenericFileDialogue<T: 'static> {
    obj: Option<T>,
    #[cfg(target_arch = "wasm32")]
    wasm_task: FileDialogueWasmTask<T>,
}

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
            self.obj = self.wasm_task.take_output().map(|x| x.unwrap());
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
        None => return Err(Box::new(FileDialogueError::default())),
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
            let file = get_bytes(
                AsyncFileDialog::new()
                    .add_filter("text", $extensions)
                    .set_directory("./")
                    .pick_file()
                    .await,
            )
            .await?;
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
