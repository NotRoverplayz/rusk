mod app;
mod file_list;
mod filesystem;
mod navigation;
mod sidebar;
mod theme;

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("RUSK Explorer")
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([760.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RUSK Explorer",
        options,
        Box::new(|context| Ok(Box::new(app::ExplorerApp::new(context)))),
    )
}
