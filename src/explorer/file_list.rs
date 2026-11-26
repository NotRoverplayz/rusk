use super::{filesystem::FileEntry, theme};
use chrono::{DateTime, Local};
use eframe::egui;
use std::{cmp::Ordering, path::PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortBy {
    Name,
    Size,
    Type,
    Modified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMode {
    Details,
    Grid,
}

pub enum Activation {
    Navigate(PathBuf),
    OpenFile(PathBuf),
}

pub fn sort_entries(entries: &mut [FileEntry], sort: SortBy, descending: bool) {
    entries.sort_by(|left, right| {
        let folder_order = right.is_dir.cmp(&left.is_dir);
        if folder_order != Ordering::Equal {
            return folder_order;
        }

        let ordering = match sort {
            SortBy::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
            SortBy::Size => left.size.cmp(&right.size),
            SortBy::Type => left.kind.to_lowercase().cmp(&right.kind.to_lowercase()),
            SortBy::Modified => left.modified.cmp(&right.modified),
        };
        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

pub fn render(
    ui: &mut egui::Ui,
    entries: &[FileEntry],
    search: &str,
    selected: &mut Option<PathBuf>,
    view: ViewMode,
) -> Option<Activation> {
    let query = search.to_lowercase();
    let matches = entries
        .iter()
        .filter(|entry| entry.name.to_lowercase().contains(&query))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(
                egui::RichText::new(if search.is_empty() {
                    "This folder is empty"
                } else {
                    "No matching files or folders"
                })
                .color(theme::MUTED),
            );
        });
        return None;
    }

    match view {
        ViewMode::Details => render_details(ui, &matches, selected),
        ViewMode::Grid => render_grid(ui, &matches, selected),
    }
}

fn render_details(
    ui: &mut egui::Ui,
    entries: &[&FileEntry],
    selected: &mut Option<PathBuf>,
) -> Option<Activation> {
    ui.horizontal(|ui| {
        ui.add_sized(
            [(ui.available_width() - 360.0).max(120.0), 24.0],
            egui::Label::new(egui::RichText::new("Name").size(11.0).color(theme::MUTED)),
        );
        ui.add_sized(
            [76.0, 24.0],
            egui::Label::new(egui::RichText::new("Size").size(11.0).color(theme::MUTED)),
        );
        ui.add_sized(
            [105.0, 24.0],
            egui::Label::new(egui::RichText::new("Type").size(11.0).color(theme::MUTED)),
        );
        ui.add_sized(
            [155.0, 24.0],
            egui::Label::new(
                egui::RichText::new("Date modified")
                    .size(11.0)
                    .color(theme::MUTED),
            ),
        );
    });
    ui.separator();

    let mut activated = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in entries {
            let name_width = (ui.available_width() - 360.0).max(120.0);
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [name_width, 28.0],
                    egui::SelectableLabel::new(
                        selected.as_ref() == Some(&entry.path),
                        format!("{}  {}", icon(entry), entry.name),
                    ),
                );
                ui.add_sized(
                    [76.0, 24.0],
                    egui::Label::new(
                        egui::RichText::new(if entry.is_dir {
                            "—".to_owned()
                        } else {
                            format_size(entry.size)
                        })
                        .color(theme::MUTED),
                    ),
                );
                ui.add_sized(
                    [105.0, 24.0],
                    egui::Label::new(egui::RichText::new(&entry.kind).color(theme::MUTED)),
                );
                ui.add_sized(
                    [155.0, 24.0],
                    egui::Label::new(
                        egui::RichText::new(format_modified(entry.modified)).color(theme::MUTED),
                    ),
                );

                if response.clicked() {
                    *selected = Some(entry.path.clone());
                }
                if response.double_clicked() {
                    activated = Some(if entry.is_dir {
                        Activation::Navigate(entry.path.clone())
                    } else {
                        Activation::OpenFile(entry.path.clone())
                    });
                }
            });
        }
    });
    activated
}

fn render_grid(
    ui: &mut egui::Ui,
    entries: &[&FileEntry],
    selected: &mut Option<PathBuf>,
) -> Option<Activation> {
    let mut activated = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("file_grid")
            .num_columns(5)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (index, entry) in entries.iter().enumerate() {
                    let response = ui.add_sized(
                        [126.0, 80.0],
                        egui::SelectableLabel::new(
                            selected.as_ref() == Some(&entry.path),
                            format!("{}\n{}", icon(entry), entry.name),
                        ),
                    );
                    if response.clicked() {
                        *selected = Some(entry.path.clone());
                    }
                    if response.double_clicked() {
                        activated = Some(if entry.is_dir {
                            Activation::Navigate(entry.path.clone())
                        } else {
                            Activation::OpenFile(entry.path.clone())
                        });
                    }
                    if (index + 1) % 5 == 0 {
                        ui.end_row();
                    }
                }
            });
    });
    activated
}

fn icon(entry: &FileEntry) -> &'static str {
    if entry.is_dir {
        "▰"
    } else {
        "▤"
    }
}

pub fn format_size(size: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = size as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_modified(modified: Option<std::time::SystemTime>) -> String {
    modified
        .map(|time| {
            DateTime::<Local>::from(time)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "—".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn entry(name: &str, is_dir: bool) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            name: name.to_owned(),
            is_dir,
            size: 0,
            modified: Some(SystemTime::UNIX_EPOCH),
            kind: if is_dir { "Folder" } else { "File" }.to_owned(),
        }
    }

    #[test]
    fn folders_stay_first_when_sorting_descending() {
        let mut entries = vec![entry("a-file", false), entry("z-folder", true)];
        sort_entries(&mut entries, SortBy::Name, true);
        assert!(entries[0].is_dir);
    }
}
