use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Bookmark {
    pub title: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Visit {
    pub title: String,
    pub url: String,
    pub visited_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Settings {
    pub homepage: String,
    pub search_engine: String,
    pub theme: String,
    pub startup: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            homepage: "rusk://home".to_owned(),
            search_engine: "https://www.google.com/search?q={query}".to_owned(),
            theme: "dark".to_owned(),
            startup: "home".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub(super) struct BrowserData {
    pub settings: Settings,
    pub bookmarks: Vec<Bookmark>,
    pub history: Vec<Visit>,
}

pub(super) struct Storage {
    path: PathBuf,
}

impl Storage {
    pub fn new() -> io::Result<Self> {
        let base = env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
            .unwrap_or(env::current_dir()?);
        let directory = base.join("Rusk").join("Browser");
        fs::create_dir_all(&directory)?;
        Ok(Self {
            path: directory.join("browser-data.json"),
        })
    }

    pub fn load(&self) -> io::Result<BrowserData> {
        if !self.path.exists() {
            return Ok(BrowserData::default());
        }
        let bytes = fs::read(&self.path)?;
        serde_json::from_slice(&bytes).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Could not parse {}: {error}", self.path.display()),
            )
        })
    }

    pub fn save(&self, data: &BrowserData) -> io::Result<()> {
        let bytes = serde_json::to_vec_pretty(data)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = self
            .path
            .with_file_name(format!("browser-data-{}.json.tmp", std::process::id()));
        fs::write(&temporary, bytes)?;
        replace_file(&temporary, &self.path)
    }
}

fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(source, destination)
}

pub(super) fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
