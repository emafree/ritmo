//! User interface prompts and messaging
//!
//! This module provides soft messaging and user prompts for interactive
//! operations like downloading binaries and confirming repairs.

use std::io::{self, Write};

/// Ask the user if they want to download binaries from GitHub
pub fn ask_download_binaries() -> bool {
    println!();
    println!("ℹ️  The binaries are not included in your library.");
    println!("They need to be downloaded from GitHub to continue.");
    print!("\n📥 Do you want to download the binaries now? (y/n): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_lowercase() == "y"
}

/// Ask the user to confirm library repair
pub fn confirm_repair() -> bool {
    println!();
    println!("ℹ️  Your library will be repaired.");
    println!("All your books will be preserved.");
    print!("\n🔧 Continue? (y/n): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    input.trim().to_lowercase() == "y"
}

#[cfg(test)]
mod tests {
    // Note: These functions require user input, so they're hard to unit test
    // They should be tested manually or with integration tests
}
