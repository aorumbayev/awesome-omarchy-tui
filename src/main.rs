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

const LOGO: &str = r#"
   ▄████████  ▄█     █▄     ████████     ▄████████   ▄██████▄    ▄▄▄▄███▄▄▄▄      ████████
  ███    ███ ███     ███   ███    ███   ███    ███  ███    ███ ▄██▀▀▀███▀▀▀██▄   ███    ███
  ███    ███ ███     ███   ███    █▀    ███    █▀   ███    ███ ███   ███   ███   ███    █▀
  ███    ███ ███     ███   ███▄▄▄       ███         ███    ███ ███   ███   ███  ▄███▄▄▄
▀███████████ ███     ███   ███▀▀▀       ██████████  ███    ███ ███   ███   ███ ▀▀███▀▀▀
  ███    ███ ███     ███   ███    █▄           ███  ███    ███ ███   ███   ███   ███    █▄
  ███    ███ ███ ▄█▄ ███   ███    ███    ▄█    ███  ███    ███ ███   ███   ███   ███    ███
  ███    █▀   ▀███▀███▀    ██████████  ▄█████████▀   ▀██████▀   ▀█   ███   █▀    ██████████

                 ▄▄▄
 ▄█████▄    ▄███████████▄    ▄███████   ▄███████   ▄███████   ▄█   █▄    ▄█   █▄
███   ███  ███   ███   ███  ███   ███  ███   ███  ███   ███  ███   ███  ███   ███
███   ███  ███   ███   ███  ███   ███  ███   ███  ███   █▀   ███   ███  ███   ███
███   ███  ███   ███   ███ ▄███▄▄▄███ ▄███▄▄▄██▀  ███       ▄███▄▄▄███▄ ███▄▄▄███
███   ███  ███   ███   ███ ▀███▀▀▀███ ▀███▀▀▀▀    ███      ▀▀███▀▀▀███  ▀▀▀▀▀▀███
███   ███  ███   ███   ███  ███   ███ ██████████  ███   █▄   ███   ███  ▄██   ███
███   ███  ███   ███   ███  ███   ███  ███   ███  ███   ███  ███   ███  ███   ███
 ▀█████▀    ▀█   ███   █▀   ███   █▀   ███   ███  ███████▀   ███   █▀    ▀█████▀
                                       ███   █▀
"#;

#[derive(Parser)]
#[command(name = "awsomarchy")]
#[command(version = VERSION, about = "A beautiful terminal UI for browsing the awesome-omarchy repository", long_about = None, disable_version_flag = true, disable_help_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Update awsomarchy to the latest version
    #[cfg(feature = "updater")]
    Update {
        /// Force update even if already on latest version
        #[arg(long)]
        force: bool,
    },
    /// Show version information
    Version,
    /// Show help information
    Help,
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
            println!("{}", LOGO);
            println!("awsomarchy v{}", VERSION);
            println!("A beautiful terminal UI for browsing the awesome-omarchy repository");
            println!("\nFeatures:");
            println!("• ⚡ Lightning fast with intelligent caching");
            println!("• 🧭 Intuitive vim-like navigation");
            println!("• 🔍 Powerful real-time search");
            println!("• 📱 Responsive sidebar layout");
            println!("• 🔄 Self-updating capability");
            return Ok(());
        }
        Some(Commands::Help) => {
            println!("{}", LOGO);
            println!("awsomarchy v{}", VERSION);
            println!("A beautiful terminal UI for browsing the awesome-omarchy repository\n");

            println!("USAGE:");
            println!("    awsomarchy [COMMAND]\n");

            println!("COMMANDS:");
            #[cfg(feature = "updater")]
            println!("    update     Update to the latest version");
            println!("    version    Show version information");
            println!("    help       Show this help message\n");

            println!("NAVIGATION (in TUI mode):");
            println!("    h/l        Switch between sidebar and content");
            println!("    j/k        Navigate within current area");
            println!("    /          Open search");
            println!("    Enter      Open repository (from search)");
            println!("    G          Open awesome-omarchy on GitHub");
            println!("    R          Reload content");
            println!("    Q          Quit application");

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
    println!("🔄 Checking for updates...");

    let current_version = env!("CARGO_PKG_VERSION");
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("aorumbayev")
        .repo_name("awesome-omarchy-tui")
        .build()?
        .fetch()?;

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
        println!("⬇️  Downloading and installing update...");

        let status = self_update::backends::github::Update::configure()
            .repo_owner("aorumbayev")
            .repo_name("awesome-omarchy-tui")
            .bin_name("awsomarchy")
            .show_download_progress(true)
            .current_version(current_version)
            .build()?
            .update()?;

        println!("✅ Update completed successfully!");
        println!("📋 Status: {}", status.version());
    } else {
        println!("❌ No releases found");
    }

    Ok(())
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
