use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Clone, Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub kind: String,
}

impl FileEntry {
    fn from_path(path: PathBuf, metadata: fs::Metadata) -> Self {
        let is_dir = metadata.is_dir();
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        let kind = if is_dir {
            "Folder".to_owned()
        } else {
            path.extension()
                .map(|extension| format!("{} file", extension.to_string_lossy().to_uppercase()))
                .unwrap_or_else(|| "File".to_owned())
        };

        Self {
            path,
            name,
            is_dir,
            size: if is_dir { 0 } else { metadata.len() },
            modified: metadata.modified().ok(),
            kind,
        }
    }
}

pub fn list_directory(path: &Path) -> io::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for result in fs::read_dir(path)? {
        let Ok(entry) = result else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        entries.push(FileEntry::from_path(entry.path(), metadata));
    }
    Ok(entries)
}

pub fn list_drives() -> Vec<FileEntry> {
    #[cfg(windows)]
    let roots = ('A'..='Z')
        .map(|letter| PathBuf::from(format!("{letter}:\\")).into_boxed_path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    #[cfg(not(windows))]
    let roots = vec![PathBuf::from("/").into_boxed_path()];

    roots
        .into_iter()
        .filter_map(|path| {
            let path = path.into_path_buf();
            fs::metadata(&path)
                .ok()
                .map(|metadata| FileEntry::from_path(path, metadata))
        })
        .collect()
}

pub fn open_file(path: &Path) -> io::Result<()> {
    #[cfg(windows)]
    let result = std::process::Command::new("explorer.exe").arg(path).spawn();

    #[cfg(not(windows))]
    let result = std::process::Command::new("xdg-open").arg(path).spawn();

    result.map(|_| ())
}
