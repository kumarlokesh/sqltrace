//! Integration tests for the TUI functionality

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use serde_json::json;
use sqltrace_rs::{
    db::models::plan::{ExecutionPlan, PlanNode},
    ui::{App, PlanNodeUI, PlanTree},
};
use std::{io, time::Duration};

/// Helper function to create a test terminal
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Helper function to clean up the terminal
fn cleanup_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Helper to send key events to the application
async fn send_key(app: &mut App, key: KeyCode) {
    app.on_key(key);
    // Small delay to allow the app to process the event
    tokio::time::sleep(Duration::from_millis(10)).await;
}

// Helper function to create a simple test plan
fn create_test_plan() -> ExecutionPlan {
    // Create a simple plan with two nodes
    let child_node = PlanNode {
        node_type: "Seq Scan".to_string(),
        relation_name: Some("test_table".to_string()),
        alias: Some("t".to_string()),
        startup_cost: 0.0,
        total_cost: 10.0,
        actual_time: 1.0,
        actual_rows: 100,
        actual_loops: 1,
        plans: vec![],
        extra: serde_json::Value::Object(serde_json::Map::new()),
    };

    let root_node = PlanNode {
        node_type: "Limit".to_string(),
        relation_name: None,
        alias: None,
        startup_cost: 0.0,
        total_cost: 20.0,
        actual_time: 2.0,
        actual_rows: 10,
        actual_loops: 1,
        plans: vec![child_node],
        extra: serde_json::Value::Object(serde_json::Map::new()),
    };

    ExecutionPlan {
        root: root_node,
        planning_time: 1.0,
        execution_time: 3.0,
    }
}

#[tokio::test]
async fn test_arrow_key_navigation() -> Result<()> {
    // Skip UI tests in CI for now as they require a display
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    // Set up test terminal
    let mut terminal = setup_terminal()?;

    // Create test app
    let mut app = App::new();

    // Create a test plan
    let test_plan = create_test_plan();
    app.plan = Some(serde_json::to_value(&test_plan)?);

    // Build the plan tree UI
    let mut plan_tree = PlanTree::default();
    sqltrace_rs::ui::build_plan_tree_ui(&test_plan.root, &mut plan_tree, 0, None);
    app.plan_tree = plan_tree;

    // Initial selection should be None
    assert_eq!(app.selected_node, None, "Initial selection should be None");

    // Send down arrow key to select the first node
    app.on_key(KeyCode::Down);
    assert!(
        app.selected_node.is_some(),
        "Should have selected the first node"
    );

    let first_selected = app.selected_node.unwrap();

    // If there are multiple nodes, test navigation
    if app.plan_tree.nodes.len() > 1 {
        // Move down to next node
        app.on_key(KeyCode::Down);
        assert_ne!(
            app.selected_node.unwrap(),
            first_selected,
            "Selection should have moved to the next node"
        );

        // Move back up
        app.on_key(KeyCode::Up);
        assert_eq!(
            app.selected_node.unwrap(),
            first_selected,
            "Selection should have moved back to the first node"
        );
    }

    // Clean up
    cleanup_terminal(terminal)?;
    Ok(())
}

#[tokio::test]
async fn test_expand_collapse() -> Result<()> {
    // Skip UI tests in CI for now as they require a display
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    // Set up test terminal
    let mut terminal = setup_terminal()?;

    // Create test app
    let mut app = App::new();

    // Create a test plan with a node that has children
    let test_plan = create_test_plan();
    app.plan = Some(serde_json::to_value(&test_plan)?);

    // Build the plan tree UI
    let mut plan_tree = PlanTree::default();
    sqltrace_rs::ui::build_plan_tree_ui(&test_plan.root, &mut plan_tree, 0, None);
    app.plan_tree = plan_tree;

    // Select the first node
    app.selected_node = Some(0);

    // The first node should have children (from our test plan)
    assert!(
        !app.plan_tree.nodes.is_empty(),
        "Test plan should have nodes"
    );

    if !app.plan_tree.nodes[0].children.is_empty() {
        // Test expanding the node by sending right arrow key
        app.on_key(KeyCode::Right);
        assert!(
            app.plan_tree.nodes[0].expanded,
            "Node should be expanded after right arrow key"
        );

        // Test collapsing the node by sending left arrow key
        app.on_key(KeyCode::Left);
        assert!(
            !app.plan_tree.nodes[0].expanded,
            "Node should be collapsed after left arrow key"
        );
    }

    // Clean up
    cleanup_terminal(terminal)?;
    Ok(())
}
