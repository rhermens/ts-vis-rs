use std::path::PathBuf;


pub fn find_project_root(entry: &PathBuf) -> Option<PathBuf> {
    match entry.join("package.json").exists() {
        true => Some(entry.to_path_buf()),
        false => find_project_root(
            &entry
                .parent()
                .expect("Could not find root directory")
                .to_path_buf(),
        ),
    }
}
