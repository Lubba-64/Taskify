pub struct TaskifyExtensionApp;

impl TaskifyExtensionApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[allow(clippy::allow_attributes, unused_mut)]
        let mut slf = Self {};
        slf
    }
}

impl eframe::App for TaskifyExtensionApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if frame.is_web() {
                ui.label("We are rendering from WASM");
                ui.separator();
            }
            ui.label("outside of wasm text");
        });
    }
}
