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
        if let Err(e) = verify_binary_availability(latest_version).await {
            println!("❌ Update failed: Binary artifact not available");
            println!("   {}", e);
            println!("   This usually means the release was just published and binaries are still being built.");
            println!("   Please try again in a few minutes.");
            return Err(e);
        }

        println!("✅ Binary artifact verified - proceeding with download...");
        println!("⬇️  Downloading and installing update...");
        println!("   Note: The application will restart automatically after successful update.");

        // Perform the update with enhanced error handling
        match perform_safe_update(current_version).await {
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

/// Verify that the binary artifact is available for download before attempting update
#[cfg(feature = "updater")]
async fn verify_binary_availability(version: &str) -> Result<()> {
    use anyhow::{anyhow, Context};
    use reqwest::StatusCode;
    
    // Construct the expected binary download URL
    let target = get_target_triple()?;
    let binary_name = format!("awsomarchy-{}-{}", version, target);
    let archive_name = if target.contains("windows") {
        format!("{}.zip", binary_name)
    } else {
        format!("{}.tar.gz", binary_name)
    };
    
    let download_url = format!(
        "https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v{}/{}",
        version, archive_name
    );

    println!("🔍 Verifying binary availability...");
    
    // Create HTTP client with reasonable timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent(format!("awesome-omarchy-tui/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .context("Failed to create HTTP client")?;

    // Perform HEAD request to check if artifact exists
    match client.head(&download_url).send().await {
        Ok(response) => {
            match response.status() {
                StatusCode::OK => {
                    // Additional verification: check content length
                    if let Some(content_length) = response.headers().get("content-length") {
                        if let Ok(length_str) = content_length.to_str() {
                            if let Ok(length) = length_str.parse::<u64>() {
                                if length < 1000 {  // Suspiciously small for a binary
                                    return Err(anyhow!(
                                        "Binary artifact appears to be incomplete (size: {} bytes). Please try again later.",
                                        length
                                    ));
                                }
                                println!("✅ Binary verified ({} bytes)", format_bytes(length));
                            }
                        }
                    }
                    Ok(())
                }
                StatusCode::NOT_FOUND => {
                    Err(anyhow!(
                        "Binary artifact not found at expected location.\n   Expected: {}\n   This usually means the CI/CD pipeline is still building the release artifacts.",
                        download_url
                    ))
                }
                status => {
                    Err(anyhow!(
                        "Binary artifact verification failed with status: {} ({})\n   URL: {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or("Unknown"),
                        download_url
                    ))
                }
            }
        }
        Err(e) => {
            Err(anyhow!(
                "Failed to verify binary availability: {}\n   This could be due to network issues or the artifact not being ready yet.",
                e
            ))
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

/// Perform the actual update with enhanced error handling and recovery
#[cfg(feature = "updater")]
async fn perform_safe_update(current_version: &str) -> Result<self_update::Status> {
    use anyhow::{anyhow, Context};

    // Create the updater with retries and better error messages
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("aorumbayev")
        .repo_name("awesome-omarchy-tui")
        .bin_name("awsomarchy")
        .show_download_progress(true)
        .current_version(current_version)
        .no_confirm(true)  // Don't prompt for confirmation in CLI mode
        .build()
        .context("Failed to configure updater")?;

    // Attempt the update with proper error handling
    match updater.update() {
        Ok(status) => {
            // Update successful - the process should be replaced
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
                    Original error: {}", e
                ))
            } else if error_msg.contains("No such file or directory") || error_msg.contains("cannot find") {
                Err(anyhow!(
                    "Binary replacement failed - file system error.\n   \
                    This might be due to antivirus software or file system permissions.\n   \
                    Original error: {}", e
                ))
            } else if error_msg.contains("download") {
                Err(anyhow!(
                    "Download failed during update.\n   \
                    Please check your internet connection and try again.\n   \
                    If the problem persists, the release artifacts might not be ready yet.\n   \
                    Original error: {}", e
                ))
            } else {
                // For any other error, provide the original error with helpful context
                Err(anyhow!(
                    "Update failed with an unexpected error.\n   \
                    Your current installation should remain functional.\n   \
                    If this persists, please report this issue on GitHub.\n   \
                    Original error: {}", e
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
