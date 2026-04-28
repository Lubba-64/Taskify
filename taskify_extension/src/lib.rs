#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
mod wrap_app;
#[cfg(target_arch = "wasm32")]
pub use web::*;
#[cfg(target_arch = "wasm32")]
pub use wrap_app::TaskifyExtensionApp;
