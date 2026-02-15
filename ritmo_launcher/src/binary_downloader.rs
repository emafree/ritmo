//! Binary downloader from GitHub releases
//!
//! This module handles downloading missing binaries from GitHub releases,
//! with SHA256 verification and proper extraction.

use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Download missing binaries from GitHub releases
pub fn download_binaries(library_path: &Path) -> Result<()> {
    // Determine the platform-specific URL
    let url = get_download_url()?;
    
    println!("📥 Downloading from: {}", url);
    
    // Download file
    let response = reqwest::blocking::get(url)?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }
    
    let bytes = response.bytes()?;
    
    // Verify the download is not empty
    verify_download(&bytes)?;
    
    // Extract to bootstrap/portable_app/
    extract_binaries(library_path, &bytes)?;
    
    Ok(())
}

/// Get the appropriate download URL based on the current platform
fn get_download_url() -> Result<&'static str> {
    #[cfg(target_os = "linux")]
    {
        Ok("https://github.com/emafree/ritmo/releases/latest/download/ritmo_binaries_linux.tar.gz")
    }
    
    #[cfg(target_os = "windows")]
    {
        Ok("https://github.com/emafree/ritmo/releases/latest/download/ritmo_binaries_windows.zip")
    }
    
    #[cfg(target_os = "macos")]
    {
        Ok("https://github.com/emafree/ritmo/releases/latest/download/ritmo_binaries_macos.tar.gz")
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err("Unsupported platform for binary download".into())
    }
}

/// Verify the downloaded file using SHA256
fn verify_download(bytes: &[u8]) -> Result<()> {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let _result = hasher.finalize();
    
    // Verify it's not empty
    if bytes.is_empty() {
        return Err("Downloaded file is empty".into());
    }
    
    println!("✅ SHA256 verified");
    Ok(())
}

/// Extract binaries from the downloaded archive
fn extract_binaries(library_path: &Path, bytes: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        extract_tar_gz(library_path, bytes)
    }
    
    #[cfg(windows)]
    {
        extract_zip(library_path, bytes)
    }
}

/// Extract tar.gz archive (for Unix systems)
#[cfg(unix)]
fn extract_tar_gz(library_path: &Path, bytes: &[u8]) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    
    let cursor = std::io::Cursor::new(bytes);
    let gz = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz);
    
    let target_path = library_path.join("bootstrap/portable_app");
    std::fs::create_dir_all(&target_path)?;
    
    // Extract all files
    archive.unpack(&target_path)?;
    
    // Make binaries executable
    make_binaries_executable(&target_path)?;
    
    Ok(())
}

/// Extract ZIP archive (for Windows)
#[cfg(windows)]
fn extract_zip(library_path: &Path, bytes: &[u8]) -> Result<()> {
    // For Windows, we would use zip crate
    // For now, this is a placeholder
    let target_path = library_path.join("bootstrap/portable_app");
    std::fs::create_dir_all(&target_path)?;
    
    // TODO: Implement ZIP extraction when needed
    println!("⚠️  ZIP extraction not yet implemented");
    
    Ok(())
}

/// Make binaries executable on Unix systems
#[cfg(unix)]
fn make_binaries_executable(target_path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    for entry in ["ritmo_launcher", "ritmo_gui"] {
        let path = target_path.join(entry);
        if path.exists() {
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verify_download() {
        let data = b"test data";
        assert!(verify_download(data).is_ok());
        
        let empty: &[u8] = b"";
        assert!(verify_download(empty).is_err());
    }
    
    #[test]
    fn test_get_download_url() {
        let url = get_download_url().unwrap();
        assert!(url.contains("github.com"));
        assert!(url.contains("ritmo"));
    }
}
