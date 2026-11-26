mod app;
mod storage;
mod ui;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    app::run()
}
