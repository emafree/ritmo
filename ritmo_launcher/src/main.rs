//! Ritmo Launcher - Cross-platform launcher for portable Ritmo installations
//!
//! This launcher handles:
//! - Library detection (from env var, CWD, parent dirs, or default location)
//! - Library structure verification
//! - Auto-repair of corrupted libraries (preserving books)
//! - Binary download from GitHub (user-initiated)
//! - GUI launch with proper library path
//!
//! Expected directory structure:
//! ```
//! library_root/
//! ├── ritmo_library/         # Library data (config, database, storage)
//! │   ├── config/
//! │   ├── database/
//! │   └── storage/
//! └── bootstrap/
//!     └── portable_app/
//!         ├── ritmo_launcher[.exe]  # This executable
//!         └── ritmo_gui[.exe]       # GUI executable
//! ```

mod library_verifier;
mod library_repairer;
mod binary_downloader;
mod ui;
mod config_generator;

use std::path::PathBuf;
use std::process::{Command, exit};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    println!("🚀 Ritmo Launcher");
    
    // 1. Detect library path
    let library_path = detect_library_path()?;
    println!("📚 Found library at: {}", library_path.display());
    
    // 2. Verify library structure
    let structure_valid = library_verifier::verify_library_structure(&library_path);
    let binaries_present = library_verifier::binaries_exist(&library_path);
    
    if !structure_valid || !binaries_present {
        if !structure_valid {
            println!("⚠️  Library structure incomplete or corrupted");
        }
        
        // 3. Check if binaries are missing
        if !binaries_present {
            println!("❌ Binaries not found in library");
            
            // Ask user if they want to download
            if ui::ask_download_binaries() {
                println!("📥 Downloading binaries from GitHub...");
                binary_downloader::download_binaries(&library_path)?;
                println!("✅ Binaries downloaded successfully");
            } else {
                eprintln!("❌ Cannot continue without binaries");
                eprintln!("Please download the complete ritmo.zip from:");
                eprintln!("   https://github.com/emafree/ritmo/releases");
                exit(1);
            }
        }
        
        // 4. Auto-repair library (preserves books!)
        if !structure_valid {
            println!("🔧 Repairing library...");
            library_repairer::repair_library(&library_path)?;
            println!("✅ Library repaired successfully");
        }
    }
    
    // 5. Verify books are preserved
    let book_count = library_verifier::count_books(&library_path)?;
    println!("📖 Library contains {} books", book_count);
    
    // 6. Launch GUI
    println!("🎨 Launching GUI...");
    launch_gui(&library_path)?;
    
    Ok(())
}

/// Detect library path from multiple sources
fn detect_library_path() -> Result<PathBuf> {
    // 1. Check environment variable
    if let Ok(path) = std::env::var("RITMO_LIBRARY_PATH") {
        let p = PathBuf::from(path);
        if p.join("ritmo_library").exists() {
            return Ok(p);
        }
    }
    
    // 2. Check current directory
    let cwd = std::env::current_dir()?;
    if cwd.join("ritmo_library").exists() {
        return Ok(cwd);
    }
    
    // 3. Walk up directories
    let mut current = cwd.clone();
    while current.pop() {
        if current.join("ritmo_library").exists() {
            return Ok(current);
        }
    }
    
    // 4. Try to detect from executable location (for portable installs)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(portable_app_dir) = exe_path.parent() {
            if let Some(bootstrap_dir) = portable_app_dir.parent() {
                if let Some(library_root) = bootstrap_dir.parent() {
                    if library_root.join("ritmo_library").exists() {
                        return Ok(library_root.to_path_buf());
                    }
                }
            }
        }
    }
    
    // 5. Default location
    let default = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".ritmo");
    
    if default.join("ritmo_library").exists() {
        return Ok(default);
    }
    
    Err("No Ritmo library found. Please extract the ritmo.zip file first.".into())
}

/// Launch the ritmo_gui binary with the library path
fn launch_gui(library_path: &PathBuf) -> Result<()> {
    let gui_name = if cfg!(windows) {
        "ritmo_gui.exe"
    } else {
        "ritmo_gui"
    };
    
    let gui_path = library_path.join("bootstrap/portable_app").join(gui_name);
    
    if !gui_path.exists() {
        return Err(format!(
            "GUI binary not found at: {}",
            gui_path.display()
        ).into());
    }
    
    // Execute GUI with environment variable and command line argument
    let status = Command::new(&gui_path)
        .env("RITMO_LIBRARY_PATH", library_path)
        .arg("--library-path")
        .arg(library_path.join("ritmo_library"))
        .spawn()?
        .wait()?;
    
    if !status.success() {
        return Err("GUI exited with error".into());
    }
    
    Ok(())
}
