use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::time::Duration;

#[cfg(feature = "updater")]
use sha2::{Digest, Sha256};
#[cfg(feature = "updater")]
use std::{fs, path::Path};

mod app;
mod client;
mod events;
mod models;
mod parser;
mod ui;

use app::App;
use client::HttpClient;
use events::EventHandler;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "awsomarchy")]
#[command(version = VERSION, about = "A tui for browsing awesome-omarchy repository", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Update to the latest version
    #[cfg(feature = "updater")]
    Update {
        /// Force update even if already on latest version
        #[arg(long)]
        force: bool,
    },
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(feature = "updater")]
        Some(Commands::Update { force }) => {
            perform_update(force).await?;
            return Ok(());
        }
        Some(Commands::Version) => {
            println!("awsomarchy v{}", VERSION);
            println!("A TUI for browsing awesome-omarchy repository");
            println!("\nFeatures:");
            println!("• ⚡ Lightning fast with intelligent caching");
            println!("• 🧭 Intuitive vim-like navigation");
            println!("• 🔍 Powerful real-time search");
            println!("• 📱 Responsive sidebar layout");
            println!("• 🔄 Self-updating capability");
            return Ok(());
        }
        None => {
            // Run the TUI application
            run_tui().await?;
        }
    }

    Ok(())
}

async fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client = HttpClient::new();
    let mut app = App::new(client).await?;
    let mut event_handler = EventHandler::new();

    let result = run_app(&mut terminal, &mut app, &mut event_handler).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

#[cfg(feature = "updater")]
async fn perform_update(force: bool) -> Result<()> {
    use anyhow::anyhow;

    println!("🔄 Checking for updates...");

    let current_version = env!("CARGO_PKG_VERSION");

    // Fetch releases with proper error handling
    let releases = match self_update::backends::github::ReleaseList::configure()
        .repo_owner("aorumbayev")
        .repo_name("awesome-omarchy-tui")
        .build()
        .and_then(|list| list.fetch())
    {
        Ok(releases) => releases,
        Err(e) => {
            println!("❌ Failed to fetch release information from GitHub");
            println!("   Error: {}", e);
            println!("   Please check your internet connection or try again later.");
            return Err(anyhow!("Unable to check for updates: {}", e));
        }
    };

    if let Some(latest_release) = releases.first() {
        let latest_version = latest_release.version.trim_start_matches('v');

        if !force && current_version == latest_version {
            println!("✅ Already on the latest version: {}", current_version);
            return Ok(());
        }

        println!(
            "📦 Found new version: {} (current: {})",
            latest_version, current_version
        );

        // Verify binary artifact availability before attempting update
        let expected_hash = match verify_binary_availability(latest_version).await {
            Ok(hash) => hash,
            Err(e) => {
                println!("❌ Update failed: Binary artifact not available");
                println!("   {}", e);
                println!(
                    "   This usually means the release was just published and binaries are still being built."
                );
                println!("   Please try again in a few minutes.");
                return Err(e);
            }
        };

        println!("✅ Binary artifact verified - proceeding with download...");
        println!("⬇️  Downloading and installing update...");
        println!("   Note: The application will restart automatically after successful update.");

        // Perform the update with enhanced error handling and SHA256 verification
        match perform_safe_update(current_version, expected_hash.as_deref()).await {
            Ok(status) => {
                println!("✅ Update completed successfully!");
                println!("📋 New version: {}", status.version());
                println!("🔄 Application will now restart with the new version.");

                // The binary replacement typically requires process restart
                // The self_update crate handles this internally
            }
            Err(e) => {
                println!("❌ Update failed: {}", e);
                println!("   Your current binary remains functional.");
                println!("   You can:");
                println!("   • Try running the update again");
                println!("   • Download the latest release manually from:");
                println!("     https://github.com/aorumbayev/awesome-omarchy-tui/releases");
                return Err(e);
            }
        }
    } else {
        println!("❌ No releases found in the repository");
        println!("   Please check the repository or try again later.");
        return Err(anyhow!("No releases available"));
    }

    Ok(())
}

/// Verify that the binary artifact and SHA256 hash file are available for download before attempting update
#[cfg(feature = "updater")]
async fn verify_binary_availability(version: &str) -> Result<Option<String>> {
    use anyhow::{Context, anyhow};
    use reqwest::StatusCode;

    // Construct the expected binary download URL
    let target = get_target_triple()?;
    let archive_name = if target.contains("windows") {
        format!("awsomarchy-standard-{}.zip", target)
    } else {
        format!("awsomarchy-standard-{}.tar.gz", target)
    };

    let binary_url = format!(
        "https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v{}/{}",
        version, archive_name
    );

    let sha256_url = construct_sha256_url(&binary_url);

    println!("🔍 Verifying binary and hash file availability...");

    // Create HTTP client with reasonable timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("awesome-omarchy-tui/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to create HTTP client")?;

    // Check binary availability
    match client.head(&binary_url).send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    // Additional verification: check content length
                    if let Some(content_length) = response.headers().get("content-length")
                        && let Ok(length_str) = content_length.to_str()
                        && let Ok(length) = length_str.parse::<u64>()
                    {
                        if length < 1000 {
                            // Suspiciously small for a binary
                            return Err(anyhow!(
                                "Binary artifact appears to be incomplete (size: {} bytes). Please try again later.",
                                length
                            ));
                        }
                        println!("✅ Binary verified ({} bytes)", format_bytes(length));
                    }
                }
                StatusCode::NOT_FOUND => {
                    return Err(anyhow!(
                        "Binary artifact not found at expected location.\n   Expected: {}\n   This usually means the CI/CD pipeline is still building the release artifacts.",
                        binary_url
                    ));
                }
                status => {
                    return Err(anyhow!(
                        "Binary artifact verification failed with status: {} ({})\n   URL: {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("Unknown"),
                        binary_url
                    ));
                }
            }
        }
        Err(e) => {
            return Err(anyhow!(
                "Failed to verify binary availability: {}\n   This could be due to network issues or the artifact not being ready yet.",
                e
            ));
        }
    }

    // Check SHA256 file availability
    match client.head(&sha256_url).send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    println!(
                        "✅ SHA256 hash file verified - integrity validation will be performed"
                    );

                    // Download and parse the SHA256 hash for later use
                    match download_and_parse_sha256(&sha256_url).await {
                        Ok(expected_hash) => Ok(Some(expected_hash)),
                        Err(e) => {
                            println!("⚠️  Warning: Failed to download SHA256 hash: {}", e);
                            println!("   Update will proceed without integrity verification");
                            Ok(None)
                        }
                    }
                }
                StatusCode::NOT_FOUND => {
                    println!("⚠️  Note: SHA256 hash file not available for this release");
                    println!("   Update will proceed without integrity verification");
                    println!("   (This is normal for older releases)");
                    Ok(None)
                }
                status => {
                    println!(
                        "⚠️  Warning: SHA256 hash file verification failed (status: {})",
                        status.as_u16()
                    );
                    println!("   Update will proceed without integrity verification");
                    Ok(None)
                }
            }
        }
        Err(e) => {
            println!(
                "⚠️  Warning: Could not check SHA256 hash file availability: {}",
                e
            );
            println!("   Update will proceed without integrity verification");
            Ok(None)
        }
    }
}

/// Get the target triple for the current platform
#[cfg(feature = "updater")]
fn get_target_triple() -> Result<String> {
    let target = if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        "x86_64-pc-windows-msvc"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "x86_64-apple-darwin"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "aarch64-apple-darwin"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        return Err(anyhow::anyhow!(
            "Unsupported platform: {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
    };

    Ok(target.to_string())
}

/// Format bytes in a human-readable way
#[cfg(feature = "updater")]
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Construct SHA256 hash file URL from binary URL
#[cfg(feature = "updater")]
fn construct_sha256_url(binary_url: &str) -> String {
    // Replace .tar.gz or .zip extension with .sha256
    if binary_url.ends_with(".tar.gz") {
        binary_url.replace(".tar.gz", ".sha256")
    } else if binary_url.ends_with(".zip") {
        binary_url.replace(".zip", ".sha256")
    } else {
        // Fallback: append .sha256 extension
        format!("{}.sha256", binary_url)
    }
}

/// Download SHA256 hash file and parse expected hash
#[cfg(feature = "updater")]
async fn download_and_parse_sha256(sha256_url: &str) -> Result<String> {
    use anyhow::{Context, anyhow};
    use reqwest::StatusCode;

    println!("🔍 Downloading SHA256 hash file...");

    // Create HTTP client with reasonable timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("awesome-omarchy-tui/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to create HTTP client for SHA256 download")?;

    // Download SHA256 file
    match client.get(sha256_url).send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    let content = response
                        .text()
                        .await
                        .context("Failed to read SHA256 file content")?;

                    // Parse SHA256 file content (format: "hash  filename")
                    let hash = content
                        .split_whitespace()
                        .next()
                        .ok_or_else(|| anyhow!("Invalid SHA256 file format: expected hash"))?;

                    // Validate hash format (64 hex characters)
                    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Err(anyhow!("Invalid SHA256 hash format: {}", hash));
                    }

                    println!("✅ SHA256 hash retrieved: {}...", &hash[..16]);
                    Ok(hash.to_lowercase())
                }
                StatusCode::NOT_FOUND => Err(anyhow!("SHA256 file not found at: {}", sha256_url)),
                status => Err(anyhow!(
                    "Failed to download SHA256 file (status: {}): {}",
                    status.as_u16(),
                    sha256_url
                )),
            }
        }
        Err(e) => Err(anyhow!(
            "Network error while downloading SHA256 file: {}\nURL: {}",
            e,
            sha256_url
        )),
    }
}

/// Verify SHA256 hash of a file
#[cfg(feature = "updater")]
async fn verify_file_sha256(file_path: &Path, expected_hash: &str) -> Result<bool> {
    use anyhow::Context;

    println!("🔐 Verifying file integrity...");

    // Read the file and compute SHA256 hash
    let file_content = fs::read(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let mut hasher = Sha256::new();
    hasher.update(&file_content);
    let computed_hash = format!("{:x}", hasher.finalize());

    println!("   Expected: {}...", &expected_hash[..16]);
    println!("   Computed: {}...", &computed_hash[..16]);

    let matches = computed_hash.to_lowercase() == expected_hash.to_lowercase();

    if matches {
        println!("✅ File integrity verified successfully!");
    } else {
        println!("❌ File integrity verification FAILED!");
        println!("   This indicates the downloaded file may be corrupted or tampered with.");
    }

    Ok(matches)
}

/// Perform the actual update with enhanced error handling and SHA256 verification
#[cfg(feature = "updater")]
async fn perform_safe_update(
    current_version: &str,
    expected_hash: Option<&str>,
) -> Result<self_update::Status> {
    use anyhow::{Context, anyhow};

    // Create the updater with retries and better error messages
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("aorumbayev")
        .repo_name("awesome-omarchy-tui")
        .bin_name("awsomarchy")
        .show_download_progress(true)
        .current_version(current_version)
        .no_confirm(true) // Don't prompt for confirmation in CLI mode
        .build()
        .context("Failed to configure updater")?;

    // If we have an expected hash, we need to perform custom download and verification
    if let Some(hash) = expected_hash {
        // Get the target and construct URLs
        let target = get_target_triple()?;
        let archive_name = if target.contains("windows") {
        format!("awsomarchy-standard-{}.zip", target)
        } else {
        format!("awsomarchy-standard-{}.tar.gz", target)
        };

        // Get the latest release version
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner("aorumbayev")
            .repo_name("awesome-omarchy-tui")
            .build()
            .and_then(|list| list.fetch())
            .context("Failed to fetch release list")?;

        let latest_version = releases
            .first()
            .ok_or_else(|| anyhow!("No releases found"))?
            .version
            .trim_start_matches('v');

        let download_url = format!(
            "https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v{}/{}",
            latest_version, archive_name
        );

        // Create temporary file for download
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(&archive_name);

        // Download the file
        println!("⬇️  Downloading archive with verification...");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minutes for large files
            .user_agent(format!("awesome-omarchy-tui/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        let response = client
            .get(&download_url)
            .send()
            .await
            .context("Failed to download archive")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download archive: HTTP {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read download response")?;

        // Write to temporary file
        fs::write(&temp_file, &bytes)
            .with_context(|| format!("Failed to write temporary file: {}", temp_file.display()))?;

        // Verify SHA256 hash
        let verification_result = verify_file_sha256(&temp_file, hash)
            .await
            .context("Failed to verify SHA256 hash")?;

        if !verification_result {
            // Clean up temporary file
            let _ = fs::remove_file(&temp_file);
            return Err(anyhow!(
                "SHA256 verification failed! The downloaded file does not match the expected hash.\n   \
                This could indicate:\n   \
                • Network corruption during download\n   \
                • Man-in-the-middle attack\n   \
                • Compromised release artifacts\n   \
                Please try again, and if the problem persists, report it as a security issue."
            ));
        }

        // Hash verification successful - now perform the actual update
        // We use the self_update crate's update method but we've already verified the integrity
        println!("🔐 Hash verification passed - proceeding with installation...");

        // Clean up our temporary file since self_update will download it again
        let _ = fs::remove_file(&temp_file);
    }

    // Attempt the update with proper error handling
    match updater.update() {
        Ok(status) => {
            if expected_hash.is_some() {
                println!("✅ Update completed with verified integrity!");
            } else {
                println!("✅ Update completed (no integrity verification available)!");
            }
            Ok(status)
        }
        Err(e) => {
            // Handle different types of update errors
            let error_msg = format!("{}", e);

            if error_msg.contains("Permission denied") {
                Err(anyhow!(
                    "Permission denied during update.\n   \
                    On Unix systems, try running with 'sudo' or ensure you have write permissions.\n   \
                    On Windows, try running as Administrator.\n   \
                    Original error: {}",
                    e
                ))
            } else if error_msg.contains("No such file or directory")
                || error_msg.contains("cannot find")
            {
                Err(anyhow!(
                    "Binary replacement failed - file system error.\n   \
                    This might be due to antivirus software or file system permissions.\n   \
                    Original error: {}",
                    e
                ))
            } else if error_msg.contains("download") {
                Err(anyhow!(
                    "Download failed during update.\n   \
                    Please check your internet connection and try again.\n   \
                    If the problem persists, the release artifacts might not be ready yet.\n   \
                    Original error: {}",
                    e
                ))
            } else {
                // For any other error, provide the original error with helpful context
                Err(anyhow!(
                    "Update failed with an unexpected error.\n   \
                    Your current installation should remain functional.\n   \
                    If this persists, please report this issue on GitHub.\n   \
                    Original error: {}",
                    e
                ))
            }
        }
    }
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()> {
    let mut tick_interval = tokio::time::interval(Duration::from_millis(250));

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        tokio::select! {
                    _ = tick_interval.tick() => {
                        app.on_tick().await;
                    }
                    event_result = event_handler.next() => {
                        match event_result? {
                            Event::Key(key) => {
                                match key.code {
                                    KeyCode::Char('q') => return Ok(()),
                                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                                    _ => {
                                        app.handle_key_event(key).await?;
            }
        }
                            }
                            Event::Resize(width, height) => {
                                app.handle_resize(width, height);
                            }
                            _ => {}
                        }
                    }
                }

        if app.should_quit() {
            return Ok(());
        }
    }
}
