use std::fs::create_dir_all;
use std::path::Path;
use crate::patchers::KrDiff;
use crate::utils::patch_krdir::KrPatchDir;

// ... existing code ...

impl KrDiff {
    pub fn new(source_path: String, diff_path: String, dest_path: String) -> Self {
        KrDiff { source_path, diff_path, dest_path }
    }

    pub fn apply(&mut self) -> Result<(), String> {
        self.apply_with_progress(None)
    }

    pub fn apply_with_progress(&mut self, write_bytes_cb: Option<Box<dyn FnMut(i64)>>) -> Result<(), String> {
        match self.apply_inner(write_bytes_cb) {
            Ok(()) => Ok(()),
            Err(e) => {
                let message = format!("[KrDiff::apply] Error: {}", e);
                eprintln!("{message}");
                Err(message)
            }
        }
    }

    fn apply_inner(&self, write_bytes_cb: Option<Box<dyn FnMut(i64)>>) -> Result<(), Box<dyn std::error::Error>> {
        let src = Path::new(&self.source_path);
        let diffp = Path::new(&self.diff_path);

        let dst = std::path::PathBuf::from(&self.dest_path);
        if !src.exists() || !src.is_dir() { return Err(format!("[KrDiff] Source path {} does not exist or is not a directory", src.display()).into()); }
        if !diffp.exists() || !diffp.is_file() { return Err(format!("[KrDiff] Diff file {} does not exist", diffp.display()).into()); }
        if !dst.exists() { create_dir_all(&dst)?; }

        let patcher = KrPatchDir::new(self.diff_path.clone());
        patcher.patch(src.to_str().unwrap_or(""), dst.to_str().unwrap_or(""), write_bytes_cb)?;
        Ok(())
    }
}
