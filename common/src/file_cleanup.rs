pub struct FileCleanup {
    path: std::path::PathBuf,
    committed: bool,
}

impl FileCleanup {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            committed: false,
        }
    }
    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if !self.committed && self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
