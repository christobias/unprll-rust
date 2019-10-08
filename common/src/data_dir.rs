use std::path::PathBuf;

use directories::ProjectDirs;

/// Gets the default directory for storing the blockchain db, log files, etc.
pub fn get_default_data_dir() -> PathBuf {
    let path = ProjectDirs::from("cash", "Unprll Project", "Unprll").expect("Failed to get project user directory").data_dir().to_path_buf();

    std::fs::create_dir_all(&path).unwrap_or_else(|err| {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Unexpected error when creating log directory {}", err);
        }
    });

    path
}
