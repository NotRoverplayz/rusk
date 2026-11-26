## RUSK Web (v0.4 foundation)

The existing RSML build pipeline generates browser HTML and CSS:

```powershell
cargo run -- build-web websites/index.rsml
```

RSML now has responsive grid breakpoints, CSS animation keyframes, comments, state binding hooks, and a validated browser event bridge. Rusk Web documentation and implementation status are in [docs/RUSK_WEB.md](docs/RUSK_WEB.md).

## RUSK Browser

The Windows browser executable uses Wry, Winit, and the installed WebView2 Runtime:

```powershell
cargo run --bin rusk_browser
cargo check --bin rusk_browser
cargo build --release --bin rusk_browser
```

It includes independent WebViews for tabs, URL/search navigation, browser shortcuts, bookmarks,
history, and a settings page. Browser data is stored as JSON under
`%APPDATA%\Rusk\Browser\browser-data.json`. Wry does not expose a hard-reload API, so Ctrl+Shift+R
uses the regular reload behavior.

## RUSK Explorer

The standalone desktop file manager uses egui/eframe and the Windows filesystem:

```powershell
cargo run --bin rusk_explorer
cargo check --bin rusk_explorer
cargo build --release --bin rusk_explorer
```

Explorer supports directory navigation and history, known-folder shortcuts, drive browsing,
current-folder search, name/size/type/modified sorting, details and grid views, and opening files
with their default application. It does not modify or delete files.