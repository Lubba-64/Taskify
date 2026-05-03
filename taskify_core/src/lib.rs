pub mod app;
pub mod file_dialogue_component;
pub mod get_tasks;
pub mod post_file;
pub mod task;
pub mod wasm_task;
#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;
