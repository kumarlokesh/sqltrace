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
use ratatui::{backend::CrosstermBackend, Terminal};
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

    // Create app and run event loop
    let app = ui::App::new();

    // If a query was provided as a command-line argument, set it in the app
    let mut app = if let Some(query) = args.query {
        let mut app = app;
        app.query = query;
        app
    } else {
        app
    };

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
    mut app: ui::App,
    db: Database,
) -> Result<()> {
    // Initial draw
    terminal.draw(|f| ui::draw(f, &mut app))?;

    // Main event loop
    loop {
        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle key press in the app
                app.on_key(key.code);

                // Special handling for query execution
                if key.code == KeyCode::Enter && app.input_mode == ui::InputMode::Query {
                    // Clear previous plan and tree
                    app.plan = None;
                    app.plan_tree = ui::PlanTree::default();
                    app.selected_node = None;
                    app.scroll_offset = 0;

                    // Skip if query is empty
                    if app.query.trim().is_empty() {
                        app.plan = Some(serde_json::json!({ "error": "Query cannot be empty" }));
                        continue;
                    }

                    // Validate query syntax
                    if let Err(e) = ui::validate_query(&app.query) {
                        app.plan = Some(serde_json::json!({ "error": e }));
                        continue;
                    }

                    // Execute the query and get the plan
                    match db.explain(&app.query).await {
                        Ok(plan) => {
                            // Convert the plan to JSON for storage
                            app.plan =
                                Some(serde_json::to_value(&plan).unwrap_or_else(
                                    |e| serde_json::json!({ "error": e.to_string() }),
                                ));

                            // Switch to plan mode after execution
                            app.input_mode = ui::InputMode::Plan;
                        }
                        Err(e) => {
                            app.plan = Some(serde_json::json!({ "error": e.to_string() }));
                        }
                    }
                }
            }
        }

        // Handle app state updates
        app.on_tick();

        // Draw the UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Check if we should quit
        if app.should_quit {
            return Ok(());
        }
    }
}
