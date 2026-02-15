//! Ritmo Launcher - Cross-platform launcher for portable Ritmo installations
//!
//! This launcher automatically detects the library path from the executable location
//! and launches ritmo_gui with the correct --library-path argument.
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

use std::env;
use std::process::{Command, exit};

fn main() {
    // 1. Get launcher executable path
    let launcher_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: Cannot determine launcher executable path");
            eprintln!("Details: {}", e);
            exit(1);
        }
    };

    // 2. Calculate library root (parent/parent from launcher executable)
    let portable_app_dir = match launcher_path.parent() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Cannot determine portable_app directory");
            eprintln!("Launcher path: {}", launcher_path.display());
            exit(1);
        }
    };

    let bootstrap_dir = match portable_app_dir.parent() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Cannot determine bootstrap directory");
            eprintln!("Expected: .../bootstrap/portable_app/");
            eprintln!("Actual: {}", portable_app_dir.display());
            exit(1);
        }
    };

    let library_root = match bootstrap_dir.parent() {
        Some(root) => root,
        None => {
            eprintln!("Error: Cannot determine library root directory");
            eprintln!("Expected: .../library_root/bootstrap/portable_app/");
            eprintln!("Actual: {}", bootstrap_dir.display());
            exit(1);
        }
    };

    // 3. Verify library structure exists
    let library_path = library_root.join("ritmo_library");
    if !library_path.exists() {
        eprintln!("Error: Cannot find library structure");
        eprintln!("Expected library at: {}", library_path.display());
        eprintln!("Please ensure the library is properly initialized");
        exit(1);
    }

    // Verify essential subdirectories
    let config_dir = library_path.join("config");
    let database_dir = library_path.join("database");
    
    if !config_dir.exists() || !database_dir.exists() {
        eprintln!("Warning: Library structure appears incomplete");
        eprintln!("Library path: {}", library_path.display());
        if !config_dir.exists() {
            eprintln!("  - Missing: config directory");
        }
        if !database_dir.exists() {
            eprintln!("  - Missing: database directory");
        }
        eprintln!("The library may not function correctly");
    }

    // 4. Find and execute ritmo_gui
    let ritmo_gui_name = if cfg!(windows) {
        "ritmo_gui.exe"
    } else {
        "ritmo_gui"
    };
    
    let ritmo_gui_path = portable_app_dir.join(ritmo_gui_name);
    
    if !ritmo_gui_path.exists() {
        eprintln!("Error: Cannot find ritmo_gui executable");
        eprintln!("Expected at: {}", ritmo_gui_path.display());
        eprintln!("Please ensure ritmo_gui is installed in the same directory as the launcher");
        exit(1);
    }

    // Print info (can be disabled in production)
    println!("Ritmo Launcher");
    println!("Library path: {}", library_path.display());
    println!("Launching ritmo_gui...\n");

    // Launch ritmo_gui with --library-path argument
    let status = Command::new(&ritmo_gui_path)
        .arg("--library-path")
        .arg(&library_path)
        .status();

    match status {
        Ok(exit_status) => {
            // Exit with the same code as ritmo_gui
            let code = exit_status.code().unwrap_or(1);
            exit(code);
        }
        Err(e) => {
            eprintln!("Error: Failed to launch ritmo_gui");
            eprintln!("Path: {}", ritmo_gui_path.display());
            eprintln!("Details: {}", e);
            exit(1);
        }
    }
}
