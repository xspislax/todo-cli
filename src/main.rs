mod app;
mod config;
mod models;
mod ui;
mod events;
mod calendar;
mod backend;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::app::App;
use crate::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("✗ Error loading configuration: {e}");
            eprintln!("  Using default configuration...");
            let default_config = Config::default();

            if let Err(e) = default_config.ensure_paths() {
                eprintln!("✗ Error creating default paths: {e}");
                std::process::exit(1);
            }

            default_config
        }
    };

    if let Err(e) = config.ensure_paths() {
        eprintln!("✗ Error ensuring paths exist: {e}");
        eprintln!("  Config path: {:?}", Config::get_config_path().display());
        eprintln!("  Data path: {}", config.features.data_path);
        eprintln!("  Default folder: {}", config.features.default_folder);
        std::process::exit(1);
    }

    let mut terminal = match setup_terminal() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("✗ Error setting up terminal: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(config);

    let result = run_app(&mut terminal, &mut app);

    println!("\nRestoring terminal...");
    if let Err(e) = restore_terminal(&mut terminal) {
        eprintln!("✗ Error restoring terminal: {e}");
    }

    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if events::handle_events(app)? {
            break;
        }

        app.update();

        terminal.draw(|f| ui::draw(f, app))?;

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    Ok(())
}
