//! Holds common search paths implementations
use super::config::data::PATHS_MAX;
use std::path::Path;

/// This struct holds the search paths, which can change before startup but not later.
/// An Arc<SearchPath> shall be used in order to get current changes for all modules.
pub struct SearchPath {
    paths: Vec<String>,
}

impl SearchPath {
    /// Create a controlled SearchPath from a simple vector
    pub fn new(paths: &Vec<String>) -> Self {
        // 2) clean-up and straighten
        let cleaned_paths = clean_paths(&paths);
        // 3) and then cut-off unwanted parts (more paths input than PATHS_MAX)
        let paths = cleaned_paths
            .into_iter()
            .enumerate()
            .filter(|(i, _el)| i < &PATHS_MAX)
            .map(|(_i, el)| el)
            .collect();
        Self { paths }
    }
    pub fn update(&mut self, new_vec: Vec<String>) {
        let cleaned_new = clean_paths(&new_vec);
        self.paths = cleaned_new;
    }
    pub fn read(&self) -> Vec<String> {
        self.paths.clone()
    }
    pub fn len(&self) -> usize {
        self.paths.len()
    }
}

/// Clean paths checks that paths exists and that intersection
/// paths are exluded, also down-up-climbing of existing paths
/// hierarchy are working!
///
/// E.g. given:
///     "/home/user/Music/audiobooks/E-F"
///     "/home/user/Music/audiobooks/E-F/George Orwell"
///     "/home/user/Music/audiobooks/A-D/../../audiobooks/A-D"
///     "/home/user/Music/audiobooks/A-D"
///     "/home/user/Music/audiobooks/E-F/George Orwell/Animal Farm"
///
/// will lead to:
///     "/home/user/Music/audiobooks/E-F"
///     "/home/user/Music/audiobooks/A-D"
///
///
/// todo: implement vfs then write document_test
fn clean_paths(unchecked_paths: &Vec<String>) -> Vec<String> {
    let mut checked_paths: Vec<String> = vec![];
    for unchecked in unchecked_paths {
        if let Ok(path_ok) = Path::new(unchecked).canonicalize() {
            if path_ok.is_dir() {
                if let Some(path_str) = path_ok.to_str() {
                    let mut is_add_worthy = true;
                    let path_string = path_str.to_string();
                    for checked in checked_paths.iter_mut() {
                        // check all already checked path, if there is
                        // no reason to not add it, add it.
                        let path_len = path_str.len();
                        let checked_len = checked.len();
                        if path_len < checked_len {
                            // if substring matches checked must be exchanged
                            if checked[..path_len] == path_string {
                                *checked = path_string;
                                is_add_worthy = false;
                                break;
                            }
                        } else {
                            // only add if not substring with any
                            if path_string[..checked_len] == *checked {
                                //
                                is_add_worthy = false;
                                break;
                            }
                        }
                    }
                    // add it if there is no reason not to
                    if is_add_worthy {
                        checked_paths.push(path_str.to_string());
                    }
                } else {
                    warn!(
                        "Path {:?} has some encoding problem, and will not be included in search!",
                        unchecked
                    )
                }
            } else {
                warn!("Path {:?} does not exist as directory/folder, and will not be included in search!", unchecked)
            }
        } else {
            error!(
                "Path {:?} is not a valid directory/folder, and will not be included in search!",
                unchecked
            );
        }
    }
    checked_paths
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // todo: use crate vfs for better unit tests
    static HAS_FS_UP: bool = false;
    // run: mkdir -p "/tmp/adbf/Music/audiobooks/A-D"
    //      mkdir -p "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm"
    //      mkdir -p "/tmp/adbf/audible/George Orwell/Animal Farm"
    //      mkdir -p "/tmp/adbf/audible/Philip K. Dick/Electric Dreams"
    //      mkdir -p "/tmp/adbf"
    static _TEST_DATA: [&str; 6] = [
        "/tmp/adbf/Music/audiobooks/A-D",
        "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm",
        "/tmp/adbf/audible/George Orwell/Animal Farm",
        "/tmp/adbf/audible/..",
        "/tmp/adbf/audible/Philip K. Dick/Electric Dreams",
        "/tmp/adbf",
    ];

    #[test]
    fn test_clean_paths_overlap() {
        init();

        // 1 out
        let testing_path = vec![
            "/tmp/adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F".to_string(),
                    "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
                ]
            );
        }

        // 2 out
        let testing_path = vec![
            "/tmp/adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F".to_string(),
            "/tmp/adbf/audible/George Orwell".to_string(),
        ];
        let return_value = clean_paths(&testing_path);

        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F".to_string(),
                    "/tmp/adbf/audible/George Orwell".to_string(),
                ]
            );
        }
    }

    #[test]
    fn test_clean_paths_parent() {
        init();

        // ..
        let testing_path = vec![
            "/tmp/adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/../audiobooks/E-F/George Orwell/Animal Farm".to_string(),
            "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
                    "/tmp/adbf/audible/George Orwell/Animal Farm".to_string(),
                ]
            );
        }

        // ..
        let testing_path = vec![
            "/tmp/adbf/Music/../../adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec!["/tmp/adbf/Music/audiobooks".to_string(),]
            );
        }

        // ..
        let testing_path = vec![
            "/tmp/adbf/Music/../../adbf/Music/audiobooks/A-D".to_string(),
            "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
        ];
        let return_value = clean_paths(&testing_path);
        if HAS_FS_UP {
            assert_eq!(
                return_value,
                vec![
                    "/tmp/adbf/Music/audiobooks/A-D".to_string(),
                    "/tmp/adbf/Music/audiobooks/E-F/George Orwell/Animal Farm".to_string(),
                ]
            );
        }
    }
}
