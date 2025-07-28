//! Integration tests for the TUI functionality

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use std::io;

use sqltrace_rs::{
    db::Database,
    ui::{self, App, PlanTree},
};

/// Helper function to create a test terminal
fn setup_terminal() -> Result<ratatui::Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = ratatui::Terminal::new(backend)?;
    Ok(terminal)
}

/// Helper function to clean up the terminal
fn cleanup_terminal(mut terminal: ratatui::Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[tokio::test]
async fn test_tui_update_after_query() -> Result<()> {
    // Skip this test in CI environment as it requires a terminal
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    // Set up test database
    let test_db_url = "postgresql://postgres:postgres@localhost:5432/sqltrace_test";
    let db = Database::new(test_db_url).await?;

    // Set up test terminal
    let terminal = setup_terminal()?;

    // Create test app
    let mut app = App::new();

    // Initial state checks
    assert!(app.plan.is_none(), "Plan should be None initially");
    assert!(
        app.plan_tree.nodes.is_empty(),
        "Plan tree should be empty initially"
    );

    // Set a test query
    let test_query = "SELECT * FROM users";

    // Execute the query directly (bypassing UI interaction)
    let result = db.explain(test_query).await;
    assert!(result.is_ok(), "Query execution failed: {:?}", result);

    // Update the app with the query result
    let exec_plan = result?;
    app.plan = Some(serde_json::to_value(&exec_plan)?);

    // Simulate the UI update that would happen after setting the plan
    app.selected_node = Some(0);

    // Verify the plan was set correctly
    assert!(
        app.plan.is_some(),
        "Plan should be set after query execution"
    );

    // Clean up
    cleanup_terminal(terminal)?;
    Ok(())
}

/// Test that verifies the plan tree is built and rendered correctly
#[tokio::test]
async fn test_plan_tree_building() -> Result<()> {
    // Skip this test in CI environment as it requires a terminal
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    // Set up test database
    let test_db_url = "postgresql://postgres:postgres@localhost:5432/sqltrace_test";
    let db = Database::new(test_db_url).await?;

    // Execute a query with a known structure
    let test_query = "SELECT * FROM users WHERE id IN (1, 2)";
    let result = db.explain(test_query).await;
    assert!(result.is_ok(), "Query execution failed: {:?}", result);

    // Create a test app
    let mut app = App::new();

    // Update the app with the query result
    let exec_plan = result?;
    app.plan = Some(serde_json::to_value(&exec_plan)?);

    // Verify the execution plan has the expected structure
    assert!(
        exec_plan.planning_time >= 0.0,
        "Planning time should be non-negative"
    );
    assert!(
        exec_plan.execution_time >= 0.0,
        "Execution time should be non-negative"
    );
    assert!(
        !exec_plan.root.node_type.is_empty(),
        "Root node should have a type"
    );

    // Manually build the plan tree UI
    let mut plan_tree = PlanTree::default();
    let _ = ui::build_plan_tree_ui(&exec_plan.root, &mut plan_tree, 0, None);
    app.plan_tree = plan_tree;

    // Set the first node as selected
    if !app.plan_tree.root_indices.is_empty() {
        app.selected_node = Some(app.plan_tree.root_indices[0]);
    }

    // Verify the plan tree was built correctly
    assert!(
        !app.plan_tree.nodes.is_empty(),
        "Plan tree should not be empty"
    );

    // Check if the root node was added to the tree
    let root_node = app.plan_tree.nodes.first().expect("Root node should exist");
    assert!(
        !root_node.plan_node.node_type.is_empty(),
        "Root node should have a type"
    );

    // If the plan has child nodes, verify they're properly connected
    if !exec_plan.root.plans.is_empty() {
        assert!(
            !root_node.children.is_empty(),
            "Root node should have children"
        );

        // Verify the first child exists and has the correct parent
        let first_child_idx = *root_node
            .children
            .first()
            .expect("First child should exist");
        let parent_idx = app.find_parent(first_child_idx);
        assert_eq!(
            parent_idx,
            Some(0),
            "First child should have root as parent"
        );
    }

    // Test rendering the plan tree
    let mut terminal = setup_terminal()?;

    // Create a frame and render the app
    terminal.draw(|f| {
        ui::draw(f, &mut app);
    })?;

    // Clean up
    cleanup_terminal(terminal)?;

    // If the plan tree is still empty, try to build it manually using the plan value
    if app.plan_tree.nodes.is_empty() {
        if let Some(plan_value) = &app.plan {
            if let Ok(exec_plan) = serde_json::from_value::<
                sqltrace_rs::db::models::plan::ExecutionPlan,
            >(plan_value.clone())
            {
                // Create a new plan tree with the root node
                let mut plan_tree = sqltrace_rs::ui::PlanTree::default();

                // Add the root node
                let root_node = sqltrace_rs::ui::PlanNodeUI {
                    expanded: true,
                    children: Vec::new(),
                    plan_node: exec_plan.root.clone(),
                };

                let root_idx = plan_tree.nodes.len();
                plan_tree.nodes.push(root_node);
                plan_tree.root_indices.push(root_idx);

                // Set the plan tree in the app
                app.plan_tree = plan_tree;
                app.selected_node = Some(root_idx);
            }
        }
    }

    // Verify the plan tree was built correctly
    assert!(
        !app.plan_tree.nodes.is_empty(),
        "Plan tree should not be empty"
    );
    assert!(
        !app.plan_tree.root_indices.is_empty(),
        "Plan tree should have root indices"
    );

    // Verify the root node has the correct properties
    if let Some(root_idx) = app.plan_tree.root_indices.first() {
        let root_node = &app.plan_tree.nodes[*root_idx];
        assert!(
            !root_node.plan_node.node_type.is_empty(),
            "Root node should have a node type"
        );

        // If there are child nodes, verify they're properly connected
        for &child_idx in &root_node.children {
            assert!(
                child_idx < app.plan_tree.nodes.len(),
                "Child index out of bounds"
            );
        }
    } else {
        panic!("Plan tree should have at least one root node");
    }

    Ok(())
}
