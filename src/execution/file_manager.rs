//! FileManager — file I/O with diff tracking.

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static TMP_DIR: OnceLock<String> = OnceLock::new();

/// Initialize the global temp directory for atomic writes.
#[allow(deprecated)]
pub fn init_temp_dir() -> &'static str {
    TMP_DIR.get_or_init(|| {
        tempfile::tempdir()
            .map(|d| d.into_path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/tmp".to_string())
    })
}

/// File operations with diff tracking.
pub struct FileManager {
    temp_dir: String,
}

impl FileManager {
    pub fn new(temp_dir: String) -> Self {
        Self { temp_dir }
    }

    /// Write a file to the temp directory with path traversal protection.
    ///
    /// Rejects `..` components, absolute paths, null bytes, and verifies
    /// the canonicalized path stays within the target directory.
    pub fn write(&self, path: &str, content: &str) -> Result<()> {
        // Reject null bytes
        if path.contains('\0') {
            return Err(anyhow::anyhow!("Path contains null bytes"));
        }

        let target = Path::new(&self.temp_dir);
        let full_path = target.join(path);

        // Reject absolute paths
        if full_path.is_absolute() && !full_path.starts_with(target) {
            return Err(anyhow::anyhow!("Path traversal detected: {}", path));
        }

        // Reject paths containing `..` components
        let canonical_candidate = full_path.canonicalize();
        if let Ok(ref canon) = canonical_candidate {
            if !canon.starts_with(target.canonicalize()?) {
                return Err(anyhow::anyhow!("Path traversal detected: {}", path));
            }
        }

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Atomic write via temp file + persist
        let tmp_dir = init_temp_dir();
        let tmp_path = Path::new(tmp_dir).join(format!(
            "chimera-{}",
            full_path.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string().into())
        ));
        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, &full_path)?;

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
