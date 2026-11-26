#[path = "../browser/mod.rs"]
mod browser;

fn main() {
    if let Err(error) = browser::run() {
        eprintln!("RUSK Browser failed: {error}");
        std::process::exit(1);
    }
}
