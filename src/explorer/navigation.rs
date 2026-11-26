use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    ThisPc,
    Path(PathBuf),
}

#[derive(Debug)]
pub struct Navigation {
    pub current: Location,
    back: Vec<Location>,
    forward: Vec<Location>,
}

impl Navigation {
    pub fn new(path: PathBuf) -> Self {
        Self {
            current: Location::Path(path),
            back: Vec::new(),
            forward: Vec::new(),
        }
    }

    pub fn navigate(&mut self, location: Location) {
        if self.current == location {
            return;
        }
        self.back.push(self.current.clone());
        self.current = location;
        self.forward.clear();
    }

    pub fn back(&mut self) {
        if let Some(location) = self.back.pop() {
            self.forward.push(self.current.clone());
            self.current = location;
        }
    }

    pub fn forward(&mut self) {
        if let Some(location) = self.forward.pop() {
            self.back.push(self.current.clone());
            self.current = location;
        }
    }

    pub fn up(&mut self) {
        if let Location::Path(path) = &self.current {
            if let Some(parent) = path.parent() {
                self.navigate(Location::Path(parent.to_path_buf()));
            } else {
                self.navigate(Location::ThisPc);
            }
        }
    }

    pub fn can_back(&self) -> bool {
        !self.back.is_empty()
    }

    pub fn can_forward(&self) -> bool {
        !self.forward.is_empty()
    }
}

pub fn breadcrumbs(location: &Location) -> Vec<(String, Location)> {
    match location {
        Location::ThisPc => vec![("This PC".to_owned(), Location::ThisPc)],
        Location::Path(path) => {
            let mut crumbs = Vec::new();
            let mut ancestor = Path::new(path);
            loop {
                let label = if ancestor.parent().is_none() {
                    ancestor.display().to_string()
                } else {
                    ancestor
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| ancestor.display().to_string())
                };
                crumbs.push((label, Location::Path(ancestor.to_path_buf())));
                match ancestor.parent() {
                    Some(parent) => ancestor = parent,
                    None => break,
                }
            }

            crumbs.reverse();
            crumbs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn back_and_forward_restore_locations() {
        let home = PathBuf::from("C:\\Users\\river");
        let documents = PathBuf::from("C:\\Users\\river\\Documents");
        let mut navigation = Navigation::new(home.clone());

        navigation.navigate(Location::Path(documents.clone()));
        navigation.back();
        assert_eq!(navigation.current, Location::Path(home));
        navigation.forward();
        assert_eq!(navigation.current, Location::Path(documents));
    }

    #[test]
    fn navigating_to_a_new_location_clears_forward_history() {
        let mut navigation = Navigation::new(PathBuf::from("C:\\one"));
        navigation.navigate(Location::Path(PathBuf::from("C:\\two")));
        navigation.back();
        navigation.navigate(Location::ThisPc);
        assert!(!navigation.can_forward());
    }
}
