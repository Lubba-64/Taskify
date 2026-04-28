use dotenv::dotenv;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 1024.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "egui demo app",
        options,
        Box::new(|cc| Ok(Box::new(taskify_core::app::TaskifyApp::new(cc)))),
    )?;
    Ok(())
}
