use super::navigation::Location;
use eframe::egui;
use std::{path::PathBuf, sync::OnceLock};

struct SidebarItem {
    label: &'static str,
    icon: &'static str,
    path: Option<PathBuf>,
}

pub fn render(
    ui: &mut egui::Ui,
    current: &Location,
    drives: &[super::filesystem::FileEntry],
) -> Option<Location> {
    let mut target = None;
    static ITEMS: OnceLock<Vec<SidebarItem>> = OnceLock::new();
    let items = ITEMS.get_or_init(|| {
        vec![
            SidebarItem {
                label: "Home",
                icon: "⌂",
                path: dirs::home_dir(),
            },
            SidebarItem {
                label: "Desktop",
                icon: "▣",
                path: dirs::desktop_dir(),
            },
            SidebarItem {
                label: "Downloads",
                icon: "↓",
                path: dirs::download_dir(),
            },
            SidebarItem {
                label: "Documents",
                icon: "▤",
                path: dirs::document_dir(),
            },
            SidebarItem {
                label: "Pictures",
                icon: "▧",
                path: dirs::picture_dir(),
            },
            SidebarItem {
                label: "Music",
                icon: "♫",
                path: dirs::audio_dir(),
            },
            SidebarItem {
                label: "Videos",
                icon: "▶",
                path: dirs::video_dir(),
            },
        ]
    });

    ui.add_space(10.0);
    ui.label(
        egui::RichText::new("QUICK ACCESS")
            .size(10.0)
            .color(super::theme::MUTED)
            .strong(),
    );
    ui.add_space(5.0);
    for item in items {
        if let Some(path) = &item.path {
            let location = Location::Path(path.clone());
            let response = ui.selectable_label(
                current == &location,
                format!("{}   {}", item.icon, item.label),
            );
            if response.clicked() {
                target = Some(location);
            }
        }
    }

    ui.add_space(22.0);
    ui.label(
        egui::RichText::new("LOCATIONS")
            .size(10.0)
            .color(super::theme::MUTED)
            .strong(),
    );
    ui.add_space(5.0);
    let this_pc = Location::ThisPc;
    if ui
        .selectable_label(current == &this_pc, "▣   This PC")
        .clicked()
    {
        target = Some(this_pc);
    }
    for drive in drives {
        let location = Location::Path(drive.path.clone());
        if ui
            .selectable_label(
                current == &location,
                format!("▱   {}", drive.path.display()),
            )
            .clicked()
        {
            target = Some(location);
        }
    }
    target
}
