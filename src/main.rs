mod db;
mod error;
mod ui;

use std::{io, time::Duration};

use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use tracing::info;

use crate::{
    db::Database,
    error::Result,
    ui::{draw, App},
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Database connection string (e.g., postgres://user:password@localhost:5432/dbname)
    #[clap(short, long)]
    database_url: String,

    /// SQL query to analyze
    #[clap(short, long)]
    query: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    // Initialize database connection
    let db = Database::new(&args.database_url).await?;
    info!("Connected to database");

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application
    let mut app = App::new();
    
    // Set initial query if provided
    if let Some(query) = args.query {
        app.query = query;
        // TODO: Execute query and update plan
    }

    // Main event loop
    let res = run_app(&mut terminal, app, db).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    db: Database,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char(c) => app.on_key(c),
                    KeyCode::Enter => {
                        // Execute query and update plan
                        if !app.query.is_empty() {
                            match db.explain_query(&app.query).await {
                                Ok(plan) => {
                                    app.plan = Some(plan);
                                }
                                Err(e) => {
                                    app.plan = Some(serde_json::json!({"error": e.to_string()}));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
