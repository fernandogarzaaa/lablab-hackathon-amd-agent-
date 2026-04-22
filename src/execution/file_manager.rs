//! FileManager — file I/O with diff tracking.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// File operations with diff tracking.
pub struct FileManager {
    temp_dir: String,
}

impl FileManager {
    pub fn new(temp_dir: String) -> Self {
        Self { temp_dir }
    }

    /// Write a file to the temp directory.
    pub fn write(&self, path: &str, content: &str) -> Result<()> {
        let full_path = Path::new(&self.temp_dir).join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&full_path, content)?;
        Ok(())
    }

    /// Read a file from the temp directory.
    pub fn read(&self, path: &str) -> Result<String> {
        let full_path = Path::new(&self.temp_dir).join(path);
        Ok(fs::read_to_string(&full_path)?)
    }

    /// Check if a file exists in the temp directory.
    pub fn exists(&self, path: &str) -> bool {
        Path::new(&self.temp_dir).join(path).exists()
    }

    /// Get the temp directory path.
    pub fn temp_dir(&self) -> &str {
        &self.temp_dir
    }
}
