#[path = "../explorer/mod.rs"]
mod explorer;

fn main() {
    if let Err(error) = explorer::run() {
        eprintln!("RUSK Explorer failed: {error}");
        std::process::exit(1);
    }
}
