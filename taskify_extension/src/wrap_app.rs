use log::debug;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["browser", "runtime"], js_name = sendMessage)]
    fn send_message(
        extension_id: &JsValue,
        message: &JsValue,
        options: &JsValue,
    ) -> js_sys::Promise;
}

pub struct TaskifyExtensionApp {}

fn send_runtime_message(message_type: &str) {
    let message = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &message,
        &JsValue::from_str("type"),
        &JsValue::from_str(message_type),
    );
    let _ = send_message(&JsValue::null(), &message.into(), &JsValue::null());
}

impl TaskifyExtensionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

// TODO: ADD ERROR UI
impl eframe::App for TaskifyExtensionApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if ui.button("Scan Image For Task").clicked() {
                debug!("hello");
                send_runtime_message("scan_image");
            }
            if ui.button("Scan Text For Task").clicked() {
                send_runtime_message("scan_text");
            }
            if ui.button("Scan PDF For Task").clicked() {
                send_runtime_message("scan_pdf");
            }
        });
    }
}
