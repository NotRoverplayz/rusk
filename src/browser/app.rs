use super::{
    storage::{timestamp, Bookmark, BrowserData, Settings, Storage, Visit},
    ui,
};
use serde_json::{json, Value};
use std::{cell::Cell, error::Error, process::Command, rc::Rc};
use url::Url;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Fullscreen, Window, WindowId},
};
use wry::{dpi::Position, PageLoadEvent, Rect, WebView, WebViewBuilder};

const TOOLBAR_HEIGHT: f64 = 78.0;
const EXPANDED_TOOLBAR_HEIGHT: f64 = 420.0;
const MAX_HISTORY: usize = 1000;

enum BrowserEvent {
    ToolbarReady,
    ShellMessage(String),
    PageMessage(u64, String, String),
    PageNavigating(u64, String),
    PageTitle(u64, String),
    PageLoaded(u64, String),
}

struct Tab {
    id: u64,
    webview: WebView,
    title: String,
    url: String,
    loading: bool,
    internal: bool,
    ipc_internal: Rc<Cell<bool>>,
}

struct Browser {
    proxy: winit::event_loop::EventLoopProxy<BrowserEvent>,
    storage: Storage,
    data: BrowserData,
    window: Option<Window>,
    toolbar: Option<WebView>,
    tabs: Vec<Tab>,
    active_tab: Option<u64>,
    next_tab_id: u64,
    modifiers: ModifiersState,
    zoom: f64,
    status: String,
    is_fullscreen: bool,
    menu_open: bool,
}

pub(super) fn run() -> Result<(), Box<dyn Error>> {
    let storage = Storage::new()?;
    let data = storage.load()?;
    let event_loop = EventLoop::<BrowserEvent>::with_user_event().build()?;
    let mut browser = Browser {
        proxy: event_loop.create_proxy(),
        storage,
        data,
        window: None,
        toolbar: None,
        tabs: Vec::new(),
        active_tab: None,
        next_tab_id: 1,
        modifiers: ModifiersState::empty(),
        zoom: 1.0,
        status: "RUSK Browser".to_owned(),
        is_fullscreen: false,
        menu_open: false,
    };
    event_loop.run_app(&mut browser)?;
    Ok(())
}

impl ApplicationHandler<BrowserEvent> for Browser {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_title("RUSK Browser")
                .with_inner_size(LogicalSize::new(1280.0, 800.0))
                .with_min_inner_size(LogicalSize::new(720.0, 480.0)),
        ) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("Could not create browser window: {error}");
                event_loop.exit();
                return;
            }
        };
        let toolbar = match WebViewBuilder::new()
            .with_bounds(rect(0.0, 0.0, 1280.0, TOOLBAR_HEIGHT))
            .with_html(ui::toolbar_html())
            .with_ipc_handler({
                let proxy = self.proxy.clone();
                move |request| {
                    if let Err(error) =
                        proxy.send_event(BrowserEvent::ShellMessage(request.body().to_owned()))
                    {
                        eprintln!("Could not dispatch browser toolbar action: {error}");
                    }
                }
            })
            .with_on_page_load_handler({
                let proxy = self.proxy.clone();
                move |event, _| {
                    if matches!(event, PageLoadEvent::Finished) {
                        if let Err(error) = proxy.send_event(BrowserEvent::ToolbarReady) {
                            eprintln!("Could not synchronize browser toolbar: {error}");
                        }
                    }
                }
            })
            .build(&window)
        {
            Ok(toolbar) => toolbar,
            Err(error) => {
                eprintln!("Could not initialize browser toolbar: {error}");
                event_loop.exit();
                return;
            }
        };
        self.window = Some(window);
        self.toolbar = Some(toolbar);

        let startup_url = if self.data.settings.startup == "homepage"
            && self.data.settings.homepage != "rusk://home"
        {
            Some(self.data.settings.homepage.clone())
        } else {
            None
        };
        self.new_tab(event_loop, startup_url);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(Window::id) != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.save_data();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => self.resize_webviews(size),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.handle_key(event.physical_key, event_loop);
            }
            _ => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: BrowserEvent) {
        match event {
            BrowserEvent::ToolbarReady => self.sync_toolbar(),
            BrowserEvent::ShellMessage(message) => {
                if let Err(error) = self.handle_message(&message, None, event_loop) {
                    self.set_status(format!("Action failed: {error}"));
                }
            }
            BrowserEvent::PageMessage(id, source, message) => {
                let is_internal = self
                    .tabs
                    .iter()
                    .find(|tab| tab.id == id)
                    .map(|tab| tab.internal && source.starts_with("about:"))
                    .unwrap_or(false);
                if is_internal {
                    if let Err(error) = self.handle_message(&message, Some(id), event_loop) {
                        self.set_status(format!("Action failed: {error}"));
                    }
                }
            }
            BrowserEvent::PageNavigating(id, url) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    tab.loading = true;
                    if url != "about:blank" {
                        tab.internal = false;
                        tab.ipc_internal.set(false);
                        tab.url = url;
                    }
                }
                self.sync_toolbar();
            }
            BrowserEvent::PageTitle(id, title) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    if !title.trim().is_empty() {
                        tab.title = title;
                        if let Some(visit) = self
                            .data
                            .history
                            .iter_mut()
                            .rev()
                            .find(|visit| visit.url == tab.url)
                        {
                            visit.title = tab.title.clone();
                        }
                    }
                }
                self.save_data();
                self.sync_toolbar();
            }
            BrowserEvent::PageLoaded(id, url) => {
                let visit = if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    tab.loading = false;
                    if url != "about:blank" {
                        tab.url = url.clone();
                        Some((url, tab.title.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some((url, title)) = visit {
                    self.add_visit(&url, &title);
                }
                self.sync_toolbar();
            }
        }
    }
}

impl Browser {
    fn new_tab(&mut self, event_loop: &ActiveEventLoop, initial_url: Option<String>) {
        let id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.saturating_add(1);
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let proxy = self.proxy.clone();
        let internal = Rc::new(Cell::new(initial_url.is_none()));
        let internal_for_ipc = internal.clone();
        let internal_for_navigation = internal.clone();

        let mut builder = WebViewBuilder::new()
            .with_bounds(self.page_bounds())
            .with_devtools(true)
            .with_ipc_handler(move |request| {
                if internal_for_ipc.get() {
                    if let Err(error) = proxy.send_event(BrowserEvent::PageMessage(
                        id,
                        request.uri().to_string(),
                        request.body().to_owned(),
                    )) {
                        eprintln!("Could not dispatch internal browser page action: {error}");
                    }
                }
            })
            .with_navigation_handler({
                let proxy = self.proxy.clone();
                move |url| {
                    if url != "about:blank" {
                        internal_for_navigation.set(false);
                    }
                    if let Err(error) =
                        proxy.send_event(BrowserEvent::PageNavigating(id, url.clone()))
                    {
                        eprintln!("Could not update browser navigation state: {error}");
                    }
                    true
                }
            })
            .with_document_title_changed_handler({
                let proxy = self.proxy.clone();
                move |title| {
                    if let Err(error) = proxy.send_event(BrowserEvent::PageTitle(id, title)) {
                        eprintln!("Could not update browser tab title: {error}");
                    }
                }
            })
            .with_on_page_load_handler({
                let proxy = self.proxy.clone();
                move |event, url| {
                    if matches!(event, PageLoadEvent::Finished) {
                        if let Err(error) = proxy.send_event(BrowserEvent::PageLoaded(id, url)) {
                            eprintln!("Could not update browser page load state: {error}");
                        }
                    }
                }
            });

        let (initial_title, is_internal) = if let Some(url) = initial_url {
            let normalized = match self.normalize_destination(&url) {
                Ok(url) => url,
                Err(error) => {
                    self.set_status(error);
                    self.sync_toolbar();
                    return;
                }
            };
            builder = builder.with_url(normalized.clone());
            (normalized, false)
        } else {
            builder = builder.with_html(ui::home_html(
                &self.data.bookmarks,
                &self.data.history,
                &self.data.settings.theme,
            ));
            ("RUSK Browser".to_owned(), true)
        };

        let webview = match builder.build(window) {
            Ok(webview) => webview,
            Err(error) => {
                self.set_status(format!("Could not create tab: {error}"));
                return;
            }
        };
        if let Err(error) = webview.zoom(self.zoom) {
            eprintln!("Could not apply page zoom: {error}");
        }
        if let Err(error) = webview.set_visible(false) {
            eprintln!("Could not hide inactive browser tab: {error}");
        }
        self.tabs.push(Tab {
            id,
            webview,
            title: initial_title.clone(),
            url: if is_internal {
                "rusk://home".to_owned()
            } else {
                initial_title
            },
            loading: !is_internal,
            internal: is_internal,
            ipc_internal: internal,
        });
        self.activate_tab(id, event_loop);
    }

    fn activate_tab(&mut self, id: u64, _event_loop: &ActiveEventLoop) {
        for tab in &self.tabs {
            if let Err(error) = tab.webview.set_visible(tab.id == id) {
                eprintln!("Could not update browser tab visibility: {error}");
            }
        }
        self.active_tab = Some(id);
        self.status = "RUSK Browser".to_owned();
        self.sync_toolbar();
    }

    fn close_tab(&mut self, id: u64, event_loop: &ActiveEventLoop) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return;
        };
        let was_active = self.active_tab == Some(id);
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_tab = None;
            self.new_tab(event_loop, None);
        } else if was_active {
            let next_index = index.min(self.tabs.len() - 1);
            self.activate_tab(self.tabs[next_index].id, event_loop);
        } else {
            self.sync_toolbar();
        }
    }

    fn resize_webviews(&self, size: PhysicalSize<u32>) {
        let scale = self
            .window
            .as_ref()
            .map(Window::scale_factor)
            .unwrap_or(1.0);
        let width = f64::from(size.width) / scale;
        let height = f64::from(size.height) / scale;
        let toolbar_height = self.toolbar_height();
        if let Some(toolbar) = &self.toolbar {
            if let Err(error) = toolbar.set_bounds(rect(0.0, 0.0, width, toolbar_height)) {
                eprintln!("Could not resize browser toolbar: {error}");
            }
        }
        let bounds = rect(
            0.0,
            toolbar_height,
            width,
            (height - toolbar_height).max(0.0),
        );
        for tab in &self.tabs {
            if let Err(error) = tab.webview.set_bounds(bounds) {
                eprintln!("Could not resize browser tab: {error}");
            }
        }
    }

    fn page_bounds(&self) -> Rect {
        let (width, height) = self
            .window
            .as_ref()
            .map(|window| {
                let scale = window.scale_factor();
                let size = window.inner_size();
                (
                    f64::from(size.width) / scale,
                    f64::from(size.height) / scale,
                )
            })
            .unwrap_or((1280.0, 800.0));
        let toolbar_height = self.toolbar_height();
        rect(
            0.0,
            toolbar_height,
            width,
            (height - toolbar_height).max(0.0),
        )
    }

    fn toolbar_height(&self) -> f64 {
        if self.menu_open {
            EXPANDED_TOOLBAR_HEIGHT
        } else {
            TOOLBAR_HEIGHT
        }
    }

    fn set_menu_open(&mut self, open: bool) {
        if self.menu_open == open {
            return;
        }
        self.menu_open = open;
        if let Some(size) = self.window.as_ref().map(Window::inner_size) {
            self.resize_webviews(size);
        }
    }

    fn handle_message(
        &mut self,
        message: &str,
        page_id: Option<u64>,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), Box<dyn Error>> {
        let command: Value = serde_json::from_str(message)?;
        let action = command
            .get("action")
            .and_then(Value::as_str)
            .ok_or("Browser action is missing its action name")?;
        let value = command.get("value").and_then(Value::as_str).unwrap_or("");
        if action == "menu_open" {
            self.set_menu_open(true);
            return Ok(());
        }
        if action == "menu_close" {
            self.set_menu_open(false);
            return Ok(());
        }
        self.set_menu_open(false);
        let active_id = page_id.or(self.active_tab);

        match action {
            "new_tab" => self.new_tab(event_loop, None),
            "close_tab" => {
                let id = value
                    .parse::<u64>()
                    .ok()
                    .or(active_id)
                    .ok_or("There is no active browser tab")?;
                self.close_tab(id, event_loop);
            }
            "switch_tab" => {
                let id = value.parse::<u64>()?;
                if self.tabs.iter().any(|tab| tab.id == id) {
                    self.activate_tab(id, event_loop);
                }
            }
            "navigate" => self.navigate(value, active_id)?,
            "back" => self.with_active_webview(active_id, |view| view.go_back())?,
            "forward" => self.with_active_webview(active_id, |view| view.go_forward())?,
            "reload" => self.with_active_webview(active_id, |view| view.reload())?,
            "stop" => {
                self.with_active_webview(active_id, |view| view.evaluate_script("window.stop()"))?;
            }
            "home" => self.go_home(active_id)?,
            "show_home" => self.show_internal_page(active_id, "home")?,
            "show_history" => self.show_internal_page(active_id, "history")?,
            "show_bookmarks" => self.show_internal_page(active_id, "bookmarks")?,
            "show_settings" => self.show_internal_page(active_id, "settings")?,
            "bookmark" => self.add_bookmark(active_id)?,
            "remove_bookmark" => {
                let index = value.parse::<usize>()?;
                if index < self.data.bookmarks.len() {
                    self.data.bookmarks.remove(index);
                    self.save_data();
                    self.refresh_internal_page(active_id)?;
                }
            }
            "save_settings" => {
                let mut settings = serde_json::from_value::<Settings>(
                    command.get("value").cloned().unwrap_or(Value::Null),
                )?;
                settings.homepage = settings.homepage.trim().to_owned();
                settings.search_engine = settings.search_engine.trim().to_owned();
                settings.theme = if settings.theme == "light" {
                    "light".to_owned()
                } else {
                    "dark".to_owned()
                };
                if settings.homepage != "rusk://home" {
                    self.normalize_destination(&settings.homepage)
                        .map_err(|error| format!("Invalid homepage: {error}"))?;
                }
                if !settings.search_engine.contains("{query}") {
                    return Err("Search URL must contain the {query} placeholder".into());
                }
                let probe = settings.search_engine.replace("{query}", "rusk-test");
                let parsed_engine =
                    Url::parse(&probe).map_err(|error| format!("Invalid search URL: {error}"))?;
                if !matches!(parsed_engine.scheme(), "http" | "https") {
                    return Err("Search URL must use HTTP or HTTPS".into());
                }
                self.data.settings = settings;
                self.save_data();
                self.apply_theme();
                self.set_status("Settings saved".to_owned());
                self.refresh_internal_page(active_id)?;
            }
            "clear_history" => {
                self.data.history.clear();
                self.save_data();
                self.refresh_internal_page(active_id)?;
            }
            "reset_settings" => {
                self.data.settings = Settings::default();
                self.save_data();
                self.refresh_internal_page(active_id)?;
            }
            "zoom_in" => self.change_zoom(0.1)?,
            "zoom_out" => self.change_zoom(-0.1)?,
            "zoom_reset" => {
                self.zoom = 1.0;
                self.apply_zoom()?;
            }
            "fullscreen" => self.toggle_fullscreen(),
            "devtools" => {
                if let Some(tab) = self.tab_by_id(active_id) {
                    tab.webview.open_devtools();
                }
            }
            "cut" => self.edit_page(active_id, "cut")?,
            "copy" => self.edit_page(active_id, "copy")?,
            "paste" => self.edit_page(active_id, "paste")?,
            "select_all" => self.edit_page(active_id, "selectAll")?,
            "new_window" => {
                Command::new(std::env::current_exe()?).spawn()?;
            }
            "exit" => event_loop.exit(),
            _ => return Err(format!("Unsupported browser action: {action}").into()),
        }
        self.sync_toolbar();
        Ok(())
    }

    fn navigate(&mut self, input: &str, active_id: Option<u64>) -> Result<(), Box<dyn Error>> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(());
        }
        if input == "rusk://home" {
            return self.show_internal_page(active_id, "home");
        }
        let url = self.normalize_destination(input)?;
        let id = active_id.ok_or("There is no active browser tab")?;
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| tab.id == id)
            .ok_or("Browser tab no longer exists")?;
        let was_internal = tab.internal;
        tab.ipc_internal.set(false);
        if let Err(error) = tab.webview.load_url(&url) {
            tab.ipc_internal.set(was_internal);
            return Err(error.into());
        }
        tab.url = url;
        tab.loading = true;
        tab.internal = false;
        self.status = "Loading page…".to_owned();
        self.sync_toolbar();
        Ok(())
    }

    fn normalize_destination(&self, input: &str) -> Result<String, String> {
        resolve_destination(input, &self.data.settings.search_engine)
    }

    fn show_internal_page(
        &mut self,
        active_id: Option<u64>,
        page: &str,
    ) -> Result<(), Box<dyn Error>> {
        let id = active_id.ok_or("There is no active browser tab")?;
        let html = match page {
            "home" => ui::home_html(
                &self.data.bookmarks,
                &self.data.history,
                &self.data.settings.theme,
            ),
            "history" => ui::history_html(&self.data.history, &self.data.settings.theme),
            "bookmarks" => ui::bookmarks_html(&self.data.bookmarks, &self.data.settings.theme),
            "settings" => ui::settings_html(&self.data.settings),
            _ => return Err(format!("Unknown internal page: {page}").into()),
        };
        let tab = self
            .tabs
            .iter_mut()
            .find(|tab| tab.id == id)
            .ok_or("Browser tab no longer exists")?;
        tab.webview.load_html(&html)?;
        tab.ipc_internal.set(true);
        tab.url = format!("rusk://{page}");
        tab.title = match page {
            "home" => "RUSK Browser",
            "history" => "History",
            "bookmarks" => "Bookmarks",
            "settings" => "Settings",
            _ => unreachable!(),
        }
        .to_owned();
        tab.loading = false;
        tab.internal = true;
        self.status = format!("RUSK · {}", tab.title);
        self.sync_toolbar();
        Ok(())
    }

    fn refresh_internal_page(&mut self, active_id: Option<u64>) -> Result<(), Box<dyn Error>> {
        let Some(id) = active_id else {
            return Ok(());
        };
        let page = self
            .tabs
            .iter()
            .find(|tab| tab.id == id && tab.internal)
            .map(|tab| tab.url.trim_start_matches("rusk://").to_owned());
        if let Some(page) = page {
            self.show_internal_page(Some(id), &page)?;
        }

        Ok(())
    }

    fn go_home(&mut self, active_id: Option<u64>) -> Result<(), Box<dyn Error>> {
        let homepage = self.data.settings.homepage.clone();
        if homepage == "rusk://home" {
            self.show_internal_page(active_id, "home")
        } else {
            self.navigate(&homepage, active_id)
        }
    }

    fn add_bookmark(&mut self, active_id: Option<u64>) -> Result<(), Box<dyn Error>> {
        let tab = self
            .tab_by_id(active_id)
            .ok_or("There is no active browser tab")?;
        if !tab.url.starts_with("http://") && !tab.url.starts_with("https://") {
            self.set_status("Open a website before adding a bookmark".to_owned());
            return Ok(());
        }
        let bookmark = Bookmark {
            title: tab.title.clone(),
            url: tab.url.clone(),
        };
        if let Some(existing) = self
            .data
            .bookmarks
            .iter_mut()
            .find(|item| item.url == bookmark.url)
        {
            *existing = bookmark;
        } else {
            self.data.bookmarks.push(bookmark);
        }
        self.save_data();
        self.set_status("Bookmark saved".to_owned());
        Ok(())
    }

    fn add_visit(&mut self, url: &str, title: &str) {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return;
        }
        self.data.history.push(Visit {
            title: if title.trim().is_empty() {
                url.to_owned()
            } else {
                title.to_owned()
            },
            url: url.to_owned(),
            visited_at: timestamp(),
        });
        if self.data.history.len() > MAX_HISTORY {
            let excess = self.data.history.len() - MAX_HISTORY;
            self.data.history.drain(0..excess);
        }
        self.save_data();
    }

    fn edit_page(&self, active_id: Option<u64>, command: &str) -> Result<(), Box<dyn Error>> {
        let script = format!("document.execCommand({})", serde_json::to_string(command)?);
        self.with_active_webview(active_id, |view| view.evaluate_script(&script))
    }

    fn change_zoom(&mut self, delta: f64) -> Result<(), Box<dyn Error>> {
        self.zoom = (self.zoom + delta).clamp(0.5, 2.0);
        self.apply_zoom()
    }

    fn apply_zoom(&mut self) -> Result<(), Box<dyn Error>> {
        for tab in &self.tabs {
            tab.webview.zoom(self.zoom)?;
        }
        self.set_status(format!("Zoom: {}%", (self.zoom * 100.0).round() as u16));
        Ok(())
    }

    fn toggle_fullscreen(&mut self) {
        let Some(window) = &self.window else {
            return;
        };
        self.is_fullscreen = !self.is_fullscreen;
        window.set_fullscreen(if self.is_fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
    }

    fn apply_theme(&self) {
        if let Some(toolbar) = &self.toolbar {
            let theme = if self.data.settings.theme == "light" {
                "light"
            } else {
                "dark"
            };
            let script = format!("document.documentElement.dataset.theme='{theme}'");
            if let Err(error) = toolbar.evaluate_script(&script) {
                eprintln!("Could not apply browser theme: {error}");
            }
        }
    }

    fn handle_key(&mut self, key: PhysicalKey, event_loop: &ActiveEventLoop) {
        let PhysicalKey::Code(key) = key else {
            return;
        };
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let shift = self.modifiers.shift_key();
        let alt = self.modifiers.alt_key();
        let action = if key == KeyCode::F11 {
            Some("fullscreen")
        } else if ctrl {
            match key {
                KeyCode::KeyT => Some("new_tab"),
                KeyCode::KeyW => Some("close_tab"),
                KeyCode::KeyR => Some("reload"),
                KeyCode::KeyL => Some("focus_address"),
                KeyCode::KeyD => Some("bookmark"),
                KeyCode::KeyH => Some("show_history"),
                KeyCode::KeyB => Some("show_bookmarks"),
                KeyCode::Tab if shift => Some("previous_tab"),
                KeyCode::Tab => Some("next_tab"),
                KeyCode::Equal | KeyCode::NumpadAdd => Some("zoom_in"),
                KeyCode::Minus | KeyCode::NumpadSubtract => Some("zoom_out"),
                KeyCode::Digit0 | KeyCode::Numpad0 => Some("zoom_reset"),
                _ => None,
            }
        } else if alt && key == KeyCode::ArrowLeft {
            Some("back")
        } else if alt && key == KeyCode::ArrowRight {
            Some("forward")
        } else {
            None
        };
        let Some(action) = action else {
            return;
        };
        if action == "focus_address" {
            if let Some(toolbar) = &self.toolbar {
                if let Err(error) = toolbar.evaluate_script(
                    "const a=document.getElementById('address');a.focus();a.select()",
                ) {
                    eprintln!("Could not focus browser address bar: {error}");
                }
            }
            return;
        }
        if action == "next_tab" || action == "previous_tab" {
            self.cycle_tab(action == "next_tab");
            return;
        }
        if let Err(error) =
            self.handle_message(&json!({"action": action}).to_string(), None, event_loop)
        {
            self.set_status(format!("Shortcut failed: {error}"));
        }
    }

    fn cycle_tab(&mut self, next: bool) {
        if self.tabs.len() < 2 {
            return;
        }
        let current = self
            .tabs
            .iter()
            .position(|tab| Some(tab.id) == self.active_tab)
            .unwrap_or(0);
        let index = if next {
            (current + 1) % self.tabs.len()
        } else {
            (current + self.tabs.len() - 1) % self.tabs.len()
        };
        if let Some(tab) = self.tabs.get(index) {
            for candidate in &self.tabs {
                if let Err(error) = candidate.webview.set_visible(candidate.id == tab.id) {
                    eprintln!("Could not switch browser tab: {error}");
                }
            }
            self.active_tab = Some(tab.id);
            self.sync_toolbar();
        }
    }

    fn with_active_webview(
        &self,
        id: Option<u64>,
        action: impl FnOnce(&WebView) -> wry::Result<()>,
    ) -> Result<(), Box<dyn Error>> {
        let tab = self.tab_by_id(id).ok_or("There is no active browser tab")?;
        action(&tab.webview)?;
        Ok(())
    }

    fn tab_by_id(&self, id: Option<u64>) -> Option<&Tab> {
        self.tabs.iter().find(|tab| Some(tab.id) == id)
    }

    fn set_status(&mut self, status: String) {
        self.status = status;
        self.sync_toolbar();
    }

    fn sync_toolbar(&self) {
        let Some(toolbar) = &self.toolbar else {
            return;
        };
        let tabs = self
            .tabs
            .iter()
            .map(|tab| {
                json!({
                    "id": tab.id,
                    "title": tab.title,
                    "active": self.active_tab == Some(tab.id),
                    "loading": tab.loading
                })
            })
            .collect::<Vec<_>>();
        let url = self
            .tab_by_id(self.active_tab)
            .map(|tab| tab.url.as_str())
            .unwrap_or("");
        let data = json!({
            "tabs": tabs,
            "url": url,
            "status": self.status,
            "theme": self.data.settings.theme,
            "loading": self.tab_by_id(self.active_tab).map(|tab| tab.loading).unwrap_or(false)
        });
        match serde_json::to_string(&data) {
            Ok(data) => {
                if let Err(error) = toolbar.evaluate_script(&format!("window.ruskUpdate({data})")) {
                    eprintln!("Could not update browser toolbar: {error}");
                }
            }
            Err(error) => eprintln!("Could not serialize browser toolbar state: {error}"),
        }
    }

    fn save_data(&self) {
        if let Err(error) = self.storage.save(&self.data) {
            eprintln!("Could not save browser data: {error}");
        }
    }
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> Rect {
    Rect {
        position: Position::Logical(LogicalPosition::new(x, y)),
        size: LogicalSize::new(width, height).into(),
    }
}

fn resolve_destination(input: &str, search_engine: &str) -> Result<String, String> {
    if input.chars().any(char::is_whitespace) {
        return Ok(build_search_url(input, search_engine));
    }
    if let Some((scheme, _)) = input.split_once(':') {
        let is_scheme = !scheme.is_empty()
            && scheme.chars().enumerate().all(|(index, ch)| {
                ch.is_ascii_alphabetic()
                    || (index > 0 && (ch.is_ascii_digit() || matches!(ch, '+' | '-' | '.')))
            });
        if is_scheme && !scheme.contains('.') && !input.starts_with("localhost:") {
            if !input.contains("://") {
                return Err("Only HTTP and HTTPS addresses are supported".to_owned());
            }
        }
    }
    let candidate = if input.contains("://") {
        input.to_owned()
    } else if input.starts_with("localhost") || input.contains('.') || input.starts_with('[') {
        format!("https://{input}")
    } else {
        return Ok(build_search_url(input, search_engine));
    };
    let url = Url::parse(&candidate).map_err(|error| format!("Invalid address: {error}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url.to_string()),
        _ => Err("Only HTTP and HTTPS addresses are supported".to_owned()),
    }
}

fn build_search_url(query: &str, search_engine: &str) -> String {
    let encoded = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    search_engine.replace("{query}", &encoded)
}

#[cfg(test)]
mod tests {
    use super::{build_search_url, resolve_destination};

    const SEARCH: &str = "https://search.example/?q={query}";

    #[test]
    fn bare_host_defaults_to_https() {
        assert_eq!(
            resolve_destination("example.com/path", SEARCH).unwrap(),
            "https://example.com/path"
        );
    }

    #[test]
    fn plain_text_is_encoded_as_a_search() {
        assert_eq!(
            resolve_destination("rusk browser", SEARCH).unwrap(),
            "https://search.example/?q=rusk+browser"
        );
        assert_eq!(
            build_search_url("a & b", SEARCH),
            "https://search.example/?q=a+%26+b"
        );
    }

    #[test]
    fn unsafe_url_schemes_are_rejected() {
        assert!(resolve_destination("javascript:alert(1)", SEARCH).is_err());
    }
}
