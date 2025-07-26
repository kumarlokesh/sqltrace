//! SQL Trace - A Terminal-Based SQL Visualizer & Advisor
//!
//! This tool helps developers understand and optimize SQL queries by visualizing
//! their execution plans in an interactive terminal interface.

#![warn(missing_docs)]

mod db;
mod error;
mod ui;

use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

// Configure logging
fn setup_logging() {
    // Only log errors in the application
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();
}

use tracing::info;

use crate::{db::Database, error::Result as SqlTraceResult};
use clap::Parser;

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
async fn main() -> SqlTraceResult<()> {
    // Set up logging first
    setup_logging();

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
    let app = if let Some(query) = args.query {
        let mut app_with_query = app;
        app_with_query.query = query;
        app_with_query
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
                // Special handling for query execution (Enter in Query mode) - check this first
                if key.code == KeyCode::Enter && app.input_mode == ui::InputMode::Query {
                    // Let the app handle the Enter key first to switch to Plan mode
                    app.on_key(key.code);
                } else {
                    // For all other keys, let the app handle them
                    let key_handled = app.on_key(key.code);

                    // If the key was handled (e.g., navigation in Plan mode), redraw and continue
                    if key_handled {
                        terminal.draw(|f| ui::draw(f, &mut app))?;
                        continue;
                    }
                }

                // Handle query execution after mode switch
                if key.code == KeyCode::Enter && app.input_mode == ui::InputMode::Plan {
                    // Clear previous plan and tree
                    app.plan = None;
                    app.plan_tree = ui::PlanTree::default();
                    app.selected_node = None;
                    app.scroll_offset = 0;

                    // Skip if query is empty
                    if app.query.trim().is_empty() {
                        app.plan = Some(serde_json::json!({ "error": "Query cannot be empty" }));
                        terminal.draw(|f| ui::draw(f, &mut app))?;
                        continue;
                    }

                    // Validate query syntax
                    if let Err(e) = ui::validate_query(&app.query) {
                        app.plan = Some(serde_json::json!({ "error": e }));
                        terminal.draw(|f| ui::draw(f, &mut app))?;
                        continue;
                    }

                    // Switch to Plan mode before executing the query
                    app.input_mode = ui::InputMode::Plan;

                    // Execute the query and get the plan
                    match db.explain(&app.query).await {
                        Ok(plan) => {
                            // Convert the plan to JSON for storage
                            match serde_json::to_value(&plan) {
                                Ok(plan_value) => {
                                    // Store the raw plan
                                    app.plan = Some(plan_value.clone());

                                    // Clear previous tree state
                                    app.plan_tree = ui::PlanTree::default();
                                    app.selected_node = None;
                                    app.scroll_offset = 0;

                                    // Parse the plan into an ExecutionPlan
                                    match serde_json::from_value::<db::models::plan::ExecutionPlan>(
                                        plan_value,
                                    ) {
                                        Ok(exec_plan) => {
                                            // Create a new plan tree
                                            let mut plan_tree = ui::PlanTree::default();

                                            // Build the plan tree UI structure
                                            ui::build_plan_tree_ui(
                                                &exec_plan.root,
                                                &mut plan_tree,
                                                0,
                                                None,
                                            );

                                            // Store the built tree
                                            app.plan_tree = plan_tree;

                                            // Select the root node if available
                                            if !app.plan_tree.root_indices.is_empty() {
                                                app.selected_node =
                                                    Some(app.plan_tree.root_indices[0]);
                                            }
                                        }
                                        Err(e) => {
                                            // If we can't parse the plan, show an error
                                            app.plan = Some(serde_json::json!({
                                                "error": format!("Failed to parse execution plan: {}", e)
                                            }));
                                        }
                                    }
                                }
                                Err(e) => {
                                    app.plan = Some(serde_json::json!({
                                        "error": format!("Failed to convert plan to JSON: {}", e)
                                    }));
                                }
                            }
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
