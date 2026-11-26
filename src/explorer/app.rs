use super::{
    file_list::{self, Activation, SortBy, ViewMode},
    filesystem::{self, FileEntry},
    navigation::{self, Location, Navigation},
    sidebar, theme,
};
use eframe::egui;
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

struct LoadResult {
    request: u64,
    result: Result<Vec<FileEntry>, String>,
}

struct DriveResult {
    entries: Vec<FileEntry>,
}

pub struct ExplorerApp {
    navigation: Navigation,
    entries: Vec<FileEntry>,
    loading: bool,
    error: Option<String>,
    search: String,
    selected: Option<PathBuf>,
    sort_by: SortBy,
    descending: bool,
    view: ViewMode,
    request_id: u64,
    load_sender: Sender<LoadResult>,
    load_receiver: Receiver<LoadResult>,
    drive_sender: Sender<DriveResult>,
    drive_receiver: Receiver<DriveResult>,
    drives: Vec<FileEntry>,
    drive_loading: bool,
    drive_error: Option<String>,
}

impl ExplorerApp {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&context.egui_ctx);
        let start = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let (load_sender, load_receiver) = mpsc::channel();
        let (drive_sender, drive_receiver) = mpsc::channel();
        let mut app = Self {
            navigation: Navigation::new(start),
            entries: Vec::new(),
            loading: false,
            error: None,
            search: String::new(),
            selected: None,
            sort_by: SortBy::Name,
            descending: false,
            view: ViewMode::Details,
            request_id: 0,
            load_sender,
            load_receiver,
            drive_sender,
            drive_receiver,
            drives: Vec::new(),
            drive_loading: true,
            drive_error: None,
        };
        app.refresh();
        let drive_sender = app.drive_sender.clone();
        match std::thread::Builder::new()
            .name("rusk-explorer-drive-scan".to_owned())
            .spawn(move || {
                let _ = drive_sender.send(DriveResult {
                    entries: filesystem::list_drives(),
                });
            }) {
            Ok(_) => {}
            Err(error) => app.drive_error = Some(format!("Could not scan drives: {error}")),
        }
        if app.drive_error.is_some() {
            app.drive_loading = false;
        }
        app
    }

    fn refresh(&mut self) {
        self.request_id = self.request_id.wrapping_add(1);
        let request = self.request_id;
        let location = self.navigation.current.clone();
        let sender = self.load_sender.clone();
        self.loading = true;
        self.error = None;
        match std::thread::Builder::new()
            .name("rusk-explorer-directory-read".to_owned())
            .spawn(move || {
                let result = read_location(&location).map_err(|error| error.to_string());
                let _ = sender.send(LoadResult { request, result });
            }) {
            Ok(_) => {}
            Err(error) => {
                self.loading = false;
                self.error = Some(format!("Could not start directory scan: {error}"));
            }
        }
    }

    fn navigate_to(&mut self, location: Location) {
        if self.navigation.current == location {
            return;
        }
        self.navigation.navigate(location);
        self.selected = None;
        self.search.clear();
        self.entries.clear();
        self.refresh();
    }

    fn poll_loads(&mut self) {
        while let Ok(message) = self.drive_receiver.try_recv() {
            self.drives = message.entries;
            self.drive_loading = false;
        }
        while let Ok(message) = self.load_receiver.try_recv() {
            if message.request != self.request_id {
                continue;
            }
            self.loading = false;
            match message.result {
                Ok(mut entries) => {
                    file_list::sort_entries(&mut entries, self.sort_by, self.descending);
                    self.entries = entries;
                    self.error = None;
                }
                Err(error) => {
                    self.entries.clear();
                    self.error = Some(error);
                }
            }
        }
    }

    fn apply_sort(&mut self, sort_by: SortBy) {
        if self.sort_by == sort_by {
            self.descending = !self.descending;
        } else {
            self.sort_by = sort_by;
            self.descending = false;
        }
        file_list::sort_entries(&mut self.entries, self.sort_by, self.descending);
    }
}

impl eframe::App for ExplorerApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_loads();

        let current = self.navigation.current.clone();
        let mut navigation_target = None;
        let mut refresh_requested = false;
        egui::TopBottomPanel::top("explorer_toolbar")
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(10, 8)),
            )
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.navigation.can_back(), egui::Button::new("←"))
                        .on_hover_text("Back")
                        .clicked()
                    {
                        self.navigation.back();
                        refresh_requested = true;
                    }
                    if ui
                        .add_enabled(self.navigation.can_forward(), egui::Button::new("→"))
                        .on_hover_text("Forward")
                        .clicked()
                    {
                        self.navigation.forward();
                        refresh_requested = true;
                    }
                    if ui
                        .add_enabled(
                            matches!(&current, Location::Path(_)),
                            egui::Button::new("↑"),
                        )
                        .on_hover_text("Up one folder")
                        .clicked()
                    {
                        self.navigation.up();
                        refresh_requested = true;
                    }
                    if ui.button("↻").on_hover_text("Refresh").clicked() {
                        refresh_requested = true;
                    }

                    ui.separator();
                    for (index, (label, location)) in
                        navigation::breadcrumbs(&current).into_iter().enumerate()
                    {
                        if index > 0 {
                            ui.label(egui::RichText::new("›").color(theme::MUTED));
                        }
                        if ui.selectable_label(current == location, label).clicked() {
                            navigation_target = Some(location);
                        }
                    }

                    ui.separator();
                    ui.add_sized(
                        [190.0, 28.0],
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Search this folder"),
                    );
                    egui::ComboBox::from_id_salt("sort_mode")
                        .selected_text(format!(
                            "Sort: {}{}",
                            sort_label(self.sort_by),
                            if self.descending { " ↓" } else { " ↑" }
                        ))
                        .show_ui(ui, |ui| {
                            for sort in [SortBy::Name, SortBy::Size, SortBy::Type, SortBy::Modified]
                            {
                                if ui
                                    .selectable_label(self.sort_by == sort, sort_label(sort))
                                    .clicked()
                                {
                                    self.apply_sort(sort);
                                }
                            }
                            if ui.button("Reverse order").clicked() {
                                self.descending = !self.descending;
                                file_list::sort_entries(
                                    &mut self.entries,
                                    self.sort_by,
                                    self.descending,
                                );
                            }
                        });
                    egui::ComboBox::from_id_salt("view_mode")
                        .selected_text(match self.view {
                            ViewMode::Details => "Details",
                            ViewMode::Grid => "Grid",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.view, ViewMode::Details, "Details");
                            ui.selectable_value(&mut self.view, ViewMode::Grid, "Grid");
                        });
                });
            });

        if let Some(target) = navigation_target.take() {
            self.navigate_to(target);
        } else if refresh_requested {
            self.refresh();
        }

        egui::SidePanel::left("explorer_sidebar")
            .resizable(true)
            .default_width(195.0)
            .width_range(155.0..=260.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::SIDEBAR)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(context, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("R")
                            .color(theme::ACCENT)
                            .strong()
                            .size(17.0),
                    );
                    ui.label(egui::RichText::new("RUSK Explorer").strong().size(14.0));
                });
                ui.separator();
                if let Some(target) = sidebar::render(ui, &self.navigation.current, &self.drives) {
                    navigation_target = Some(target);
                }
            });

        if let Some(target) = navigation_target {
            self.navigate_to(target);
        }

        egui::TopBottomPanel::bottom("explorer_status")
            .frame(
                egui::Frame::new()
                    .fill(theme::PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 6)),
            )
            .show(context, |ui| {
                ui.horizontal(|ui| {
                    let count = self.entries.len();
                    ui.label(
                        egui::RichText::new(format!(
                            "{count} items{}",
                            if self.search.is_empty() {
                                String::new()
                            } else {
                                format!(
                                    " · {} matching",
                                    self.entries
                                        .iter()
                                        .filter(|entry| entry
                                            .name
                                            .to_lowercase()
                                            .contains(&self.search.to_lowercase()))
                                        .count()
                                )
                            }
                        ))
                        .color(theme::MUTED)
                        .size(11.0),
                    );
                    if self.loading {
                        ui.label(
                            egui::RichText::new("Loading…")
                                .color(theme::MUTED)
                                .size(11.0),
                        );
                    }
                    if let Some(selected) = &self.selected {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(
                                    selected.file_name().unwrap_or_default().to_string_lossy(),
                                )
                                .color(theme::MUTED)
                                .size(11.0),
                            );
                        });
                    }
                });
            });

        egui::CentralPanel::default().show(context, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let title = match &self.navigation.current {
                    Location::ThisPc => "This PC".to_owned(),
                    Location::Path(path) => path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                };
                ui.heading(title);
                if let Location::Path(path) = &self.navigation.current {
                    ui.label(
                        egui::RichText::new(path.display().to_string())
                            .color(theme::MUTED)
                            .size(11.0),
                    );
                }
            });
            ui.separator();
            if let Some(error) = &self.error {
                ui.colored_label(
                    egui::Color32::from_rgb(235, 150, 150),
                    format!("Could not read this location: {error}"),
                );
            }
            if let Some(error) = &self.drive_error {
                ui.colored_label(egui::Color32::from_rgb(235, 150, 150), error);
            }
            if self.loading && self.entries.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new("Loading…").color(theme::MUTED));
                });
            } else if self.error.is_some() && self.entries.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("This location is unavailable").color(theme::MUTED),
                    );
                });
            } else {
                let activation = file_list::render(
                    ui,
                    &self.entries,
                    &self.search,
                    &mut self.selected,
                    self.view,
                );
                if let Some(activation) = activation {
                    match activation {
                        Activation::Navigate(path) => {
                            self.navigate_to(Location::Path(path));
                        }
                        Activation::OpenFile(path) => {
                            if let Err(error) = filesystem::open_file(&path) {
                                self.error =
                                    Some(format!("Could not open {}: {error}", path.display()));
                            }
                        }
                    }
                }
            }
        });

        if self.loading || self.drive_loading {
            context.request_repaint_after(Duration::from_millis(100));
        }
    }
}

fn read_location(location: &Location) -> std::io::Result<Vec<FileEntry>> {
    match location {
        Location::ThisPc => Ok(filesystem::list_drives()),
        Location::Path(path) => filesystem::list_directory(path),
    }
}

fn sort_label(sort: SortBy) -> &'static str {
    match sort {
        SortBy::Name => "Name",
        SortBy::Size => "Size",
        SortBy::Type => "Type",
        SortBy::Modified => "Date modified",
    }
}
