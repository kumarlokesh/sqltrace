//! Terminal User Interface (TUI) module for SQL Trace
//!
//! This module handles all terminal-based user interface components, including:
//! - Query input and validation
//! - Execution plan visualization
//! - Interactive plan exploration
//! - Error display and user feedback

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::db::models::{ExecutionPlan, PlanNode};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// Validates if the SQL query is syntactically valid
pub fn validate_query(query: &str) -> Result<(), String> {
    if query.trim().is_empty() {
        return Err("Query cannot be empty".to_string());
    }

    let dialect = PostgreSqlDialect {};
    let result = Parser::parse_sql(&dialect, query);

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("SQL Syntax Error: {}", e)),
    }
}

/// Renders a single node in the execution plan tree
fn render_plan_node(
    node: &PlanNode,
    is_selected: bool,
    is_expanded: bool,
    has_children: bool,
    level: usize,
) -> Text<'static> {
    let indent = "  ".repeat(level);
    let prefix = if has_children {
        if is_expanded {
            "[-] "
        } else {
            "[+] "
        }
    } else {
        "    "
    };

    // Node type with color coding
    let node_type = format!("{}: ", node.node_type);

    // Relation/Alias information
    let relation_info = node
        .relation_name
        .as_ref()
        .or(node.alias.as_ref())
        .map(|name| format!("on {}", name))
        .unwrap_or_default();

    // Format the cost and row information
    let details = format!(
        "(cost={:.2}..{:.2}, rows={}, time={:.2}ms{}{})",
        node.startup_cost,
        node.total_cost,
        node.actual_rows,
        node.actual_time,
        if !relation_info.is_empty() { " " } else { "" },
        relation_info
    );

    // Create the main line of the node
    let mut text = Text::from(vec![Line::from(vec![
        Span::raw(indent),
        Span::raw(prefix),
        Span::styled(
            node_type,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(details, Style::default().fg(Color::Gray)),
    ])]);

    // Add additional details if expanded
    if is_expanded {
        // Always show these basic stats
        let mut extra_info = vec![
            format!("Actual Rows: {}", node.actual_rows),
            format!("Actual Loops: {}", node.actual_loops),
            format!("Actual Time: {:.2}ms", node.actual_time),
            format!("Startup Cost: {:.2}", node.startup_cost),
            format!("Total Cost: {:.2}", node.total_cost),
        ];

        // Add any additional fields from the extra JSON
        if let Some(extra_fields) = node.extra.as_object() {
            for (key, value) in extra_fields {
                // Skip fields we're already displaying
                if ![
                    "Node Type",
                    "Relation Name",
                    "Alias",
                    "Startup Cost",
                    "Total Cost",
                    "Actual Total Time",
                    "Actual Rows",
                    "Actual Loops",
                    "Plans",
                    "Plan Rows",
                    "Plan Width",
                ]
                .contains(&key.as_str())
                {
                    extra_info.push(format!("{}: {}", key, value));
                }
            }
        }

        // Add each extra info line with proper indentation
        for info in extra_info {
            text.lines.push(Line::from(vec![
                Span::raw("  ".repeat(level + 1)),
                Span::raw("• "),
                Span::styled(info, Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // Highlight selected node
    if is_selected {
        for line in &mut text.lines {
            line.spans.iter_mut().for_each(|span| {
                *span = span
                    .clone()
                    .style(span.style.add_modifier(Modifier::REVERSED).bg(Color::Blue));
            });
        }
    }

    text
}

/// Recursively builds the plan tree UI structure
pub fn build_plan_tree_ui(
    node: &PlanNode,
    tree_ui: &mut PlanTree,
    _node_index: usize, // Not currently used, but kept for future use
    parent_index: Option<usize>,
) -> usize {
    // Create a copy of the plan node to store in the UI
    let node_copy = node.clone();

    // Create the current node UI
    let current_index = tree_ui.nodes.len();
    let children_count = node.plans.len();
    let mut child_indices = Vec::with_capacity(children_count);

    // Add the current node to the tree with a copy of the plan node
    tree_ui.nodes.push(PlanNodeUI {
        expanded: true, // Expand by default
        children: Vec::new(),
        plan_node: node_copy,
    });

    // Process children
    for child in &node.plans {
        let child_index = build_plan_tree_ui(child, tree_ui, current_index, Some(current_index));
        child_indices.push(child_index);
    }

    // Update current node with children indices
    if let Some(node_ui) = tree_ui.nodes.get_mut(current_index) {
        node_ui.children = child_indices;
    }

    // If this is a root node, add it to root_indices
    if parent_index.is_none() {
        tree_ui.root_indices.push(current_index);
    }

    current_index
}

/// Represents a UI node in the execution plan tree
///
/// This struct holds the visual state and data for a single node
/// in the execution plan tree, including its expansion state and
/// child relationships.
#[derive(Debug)]
pub struct PlanNodeUI {
    /// Whether this node is currently expanded in the UI
    pub expanded: bool,
    /// Indices of child nodes in the parent's node list
    pub children: Vec<usize>,
    /// The actual plan node data being visualized
    pub plan_node: PlanNode,
}

impl Default for PlanNodeUI {
    fn default() -> Self {
        Self {
            expanded: true,
            children: Vec::new(),
            plan_node: PlanNode {
                node_type: String::new(),
                relation_name: None,
                alias: None,
                startup_cost: 0.0,
                total_cost: 0.0,
                actual_time: 0.0,
                actual_rows: 0,
                actual_loops: 0,
                plans: Vec::new(),
                extra: serde_json::Value::Object(serde_json::Map::new()),
            },
        }
    }
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Helper function to compute a hash of the plan for change detection
fn plan_hash(plan: &serde_json::Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    plan.to_string().hash(&mut hasher);
    hasher.finish()
}

/// Represents the complete execution plan tree structure for UI rendering
///
/// This struct maintains the tree structure of the execution plan
/// along with metadata needed for efficient rendering and interaction.
#[derive(Debug, Default)]
pub struct PlanTree {
    /// All nodes in the tree, stored in a flat structure
    pub nodes: Vec<PlanNodeUI>,
    /// Indices of root nodes in the nodes vector
    pub root_indices: Vec<usize>,
    /// Hash of the last processed plan for change detection
    pub last_plan_hash: Option<u64>,
}

/// The main application state for the TUI
///
/// This struct holds all the state needed to render and update
/// the terminal user interface.
pub struct App {
    /// Whether the application should exit
    pub should_quit: bool,
    /// The current SQL query being edited or executed
    pub query: String,
    /// The current execution plan as raw JSON
    pub plan: Option<serde_json::Value>,
    /// The UI representation of the execution plan
    pub plan_tree: PlanTree,
    /// Index of the currently selected node in the plan tree
    pub selected_node: Option<usize>,
    /// Current vertical scroll offset in the plan view
    pub scroll_offset: u16,
    /// Current input mode (Query or Plan)
    pub input_mode: InputMode,
}

/// Represents the current input mode of the application
///
/// The application can be in different modes that affect
/// how keyboard input is processed.
/// Represents the current input mode of the application
///
/// The application can be in different modes that affect
/// how keyboard input is processed.
#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    /// In query mode, user can type and edit SQL queries
    Query,
    /// In plan mode, user can navigate and interact with the execution plan
    Plan,
}

impl Default for App {
    /// Creates a new App instance with default values
    ///
    /// # Returns
    /// A new `App` instance with empty state and default settings
    fn default() -> Self {
        Self {
            should_quit: false,
            query: String::new(),
            plan: None,
            plan_tree: PlanTree::default(),
            selected_node: None,
            scroll_offset: 0,
            input_mode: InputMode::Query,
        }
    }
}

impl App {
    /// Creates a new App instance with default values
    ///
    /// # Returns
    /// A new `App` instance with empty state and default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles periodic updates to the application state
    ///
    /// This method is called on each tick of the main event loop
    /// and can be used to update animations or other time-based state.
    pub fn on_tick(&mut self) {
        // Update app state on each tick if needed
    }

    /// Handles a key press event
    ///
    /// # Arguments
    /// * `key` - The key that was pressed
    ///
    /// # Returns
    /// `true` if the key was handled, `false` otherwise
    pub fn on_key(&mut self, key: KeyCode) -> bool {
        match self.input_mode {
            InputMode::Query => self.handle_query_mode(key),
            InputMode::Plan => self.handle_plan_mode(key),
        }
    }

    /// Handles key presses when in Query mode
    ///
    /// # Arguments
    /// * `key` - The key that was pressed
    ///
    /// # Returns
    /// `true` if the key was handled, `false` otherwise
    fn handle_query_mode(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                true
            }
            KeyCode::Tab => {
                self.input_mode = InputMode::Plan;
                true
            }
            KeyCode::Enter => {
                // Switch to Plan mode to show the execution plan
                // The actual query execution is handled in the main event loop
                // when it detects Enter was pressed in Query mode
                self.input_mode = InputMode::Plan;
                true
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                true
            }
            KeyCode::Backspace => {
                self.query.pop();
                true
            }
            _ => false,
        }
    }

    /// Handles key presses when in Plan mode
    ///
    /// # Arguments
    /// * `key` - The key that was pressed
    ///
    /// # Returns
    /// `true` if the key was handled, `false` otherwise
    fn handle_plan_mode(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                true
            }
            KeyCode::Tab => {
                self.input_mode = InputMode::Query;
                true
            }
            KeyCode::Down => {
                // If current node is collapsed and has children, expand it first
                if let Some(selected) = self.selected_node {
                    if let Some(node) = self.plan_tree.nodes.get(selected) {
                        if !node.expanded && !node.children.is_empty() {
                            self.expand_node();
                            return true;
                        }
                    }
                }
                self.move_selection(1);
                true
            }
            KeyCode::Up => {
                self.move_selection(-1);
                true
            }
            KeyCode::Right => {
                // If on a node with children, expand it
                if let Some(selected) = self.selected_node {
                    if let Some(node) = self.plan_tree.nodes.get(selected) {
                        if !node.children.is_empty() {
                            self.expand_node();
                            return true;
                        }
                    }
                }
                // If no children or no selection, move right to child if possible
                if let Some(selected) = self.selected_node {
                    if let Some(node) = self.plan_tree.nodes.get(selected) {
                        if !node.children.is_empty() {
                            self.selected_node = Some(node.children[0]);
                            return true;
                        }
                    }
                }
                true
            }
            KeyCode::Left => {
                self.collapse_node();
                true
            }
            KeyCode::Enter => {
                self.toggle_node();
                true
            }
            _ => false,
        }
    }

    /// Moves the current selection in the plan tree
    ///
    /// # Arguments
    /// * `delta` - The number of positions to move (positive for down, negative for up)
    fn move_selection(&mut self, delta: i32) {
        if self.plan_tree.nodes.is_empty() {
            return;
        }

        // Get the list of visible nodes (using a dummy ExecutionPlan since we don't need it for navigation)
        let dummy_plan = ExecutionPlan {
            root: PlanNode {
                node_type: "".to_string(),
                relation_name: None,
                alias: None,
                startup_cost: 0.0,
                total_cost: 0.0,
                actual_time: 0.0,
                actual_rows: 0,
                actual_loops: 0,
                plans: vec![],
                extra: serde_json::Value::Object(serde_json::Map::new()),
            },
            planning_time: 0.0,
            execution_time: 0.0,
        };

        let visible_nodes = collect_visible_nodes(
            &dummy_plan, // Not actually used in collect_visible_nodes for navigation
            &self.plan_tree,
            &self.plan_tree.root_indices,
            0,
            self.selected_node,
            self.scroll_offset,
        );

        if visible_nodes.is_empty() {
            // If nothing is selected, select the first node
            if !self.plan_tree.nodes.is_empty() {
                self.selected_node = Some(0);
            }
            return;
        }

        // If nothing is selected, select the first visible node
        if self.selected_node.is_none() && !visible_nodes.is_empty() {
            self.selected_node = Some(visible_nodes[0].0);
            return;
        }

        // Find the current selection in the visible nodes
        let current_pos = match self.selected_node {
            Some(selected) => visible_nodes
                .iter()
                .position(|(idx, _, _, _, _, _)| *idx == selected)
                .unwrap_or(0),
            None => 0,
        };

        // Calculate the new position, ensuring it's within bounds
        let new_pos = if delta > 0 {
            // If moving down and at the last node, don't move
            if current_pos >= visible_nodes.len().saturating_sub(1) {
                return;
            }
            current_pos
                .saturating_add(delta as usize)
                .min(visible_nodes.len().saturating_sub(1))
        } else {
            // If moving up and at the first node, don't move
            if current_pos == 0 {
                return;
            }
            current_pos.saturating_sub((-delta) as usize)
        };

        // Update the selection
        if new_pos < visible_nodes.len() {
            self.selected_node = Some(visible_nodes[new_pos].0);

            // Ensure the selected node is visible in the viewport
            // We'll handle scrolling in the draw function based on the selected node
        }
    }

    /// Expands the currently selected node in the plan tree
    fn expand_node(&mut self) {
        if self.plan.is_none() {
            return;
        }

        if let Some(selected) = self.selected_node {
            if let Some(node) = self.plan_tree.nodes.get_mut(selected) {
                // Only expand if the node has children and isn't already expanded
                if !node.children.is_empty() && !node.expanded {
                    node.expanded = true;

                    // If this is the first child being expanded, auto-select the first child
                    if !node.children.is_empty() {
                        self.selected_node = Some(node.children[0]);
                    }
                } else if node.expanded && !node.children.is_empty() {
                    // If already expanded, move to first child
                    self.selected_node = Some(node.children[0]);
                }
            }
        } else if !self.plan_tree.root_indices.is_empty() {
            // If nothing is selected, select and expand the first root node
            self.selected_node = Some(self.plan_tree.root_indices[0]);
            if let Some(node) = self.plan_tree.nodes.get_mut(self.plan_tree.root_indices[0]) {
                if !node.children.is_empty() {
                    node.expanded = true;
                    // Select the first child
                    self.selected_node = Some(node.children[0]);
                }
            }
        }
    }

    /// Collapses the currently selected node in the plan tree
    fn collapse_node(&mut self) {
        if let Some(selected) = self.selected_node {
            if let Some(node) = self.plan_tree.nodes.get_mut(selected) {
                // Only collapse if the node is expanded
                if node.expanded {
                    node.expanded = false;
                } else if let Some(parent_idx) = self.find_parent(selected) {
                    // If already collapsed, move to parent
                    self.selected_node = Some(parent_idx);
                }
            }
        }
    }

    /// Toggles the expansion state of the currently selected node
    fn toggle_node(&mut self) {
        if let Some(node_idx) = self.selected_node {
            if node_idx < self.plan_tree.nodes.len() {
                let node = &mut self.plan_tree.nodes[node_idx];
                node.expanded = !node.expanded;

                // If expanding, select the first child if there are any
                if node.expanded && !node.children.is_empty() {
                    self.selected_node = Some(node.children[0]);
                }
            }
        }
    }

    /// Finds the parent of a node in the plan tree
    ///
    /// # Arguments
    /// * `node_idx` - The index of the node to find the parent of
    ///
    /// # Returns
    /// The index of the parent node, or None if not found
    fn find_parent(&self, node_idx: usize) -> Option<usize> {
        // Check if this is a root node
        if self.plan_tree.root_indices.contains(&node_idx) {
            return None;
        }

        // Search through all nodes to find one that has node_idx as a child
        for (i, node) in self.plan_tree.nodes.iter().enumerate() {
            if node.children.contains(&node_idx) {
                return Some(i);
            }
        }

        None
    }
}

// / Recursively traverses the plan tree to collect visible nodes for rendering
fn collect_visible_nodes(
    _plan: &ExecutionPlan, // No longer needed as we have plan nodes in UI nodes
    tree_ui: &PlanTree,
    node_indices: &[usize],
    level: usize,
    selected_node: Option<usize>,
    _scroll_offset: u16, // Unused for now, but kept for future use
) -> Vec<(usize, PlanNode, bool, bool, usize, bool)> {
    let mut result = Vec::new();

    // Process each UI node index
    for &node_idx in node_indices {
        // Check if the node index is valid
        if node_idx >= tree_ui.nodes.len() {
            println!(
                "DEBUG - Invalid node index: {} (nodes length: {})",
                node_idx,
                tree_ui.nodes.len()
            );
            continue;
        }

        let node_ui = &tree_ui.nodes[node_idx];
        let is_selected = Some(node_idx) == selected_node;

        // Add the current node to the result with its plan node
        result.push((
            node_idx,
            node_ui.plan_node.clone(), // Use the plan node stored in the UI node
            is_selected,
            node_ui.expanded,
            level,
            !node_ui.children.is_empty(),
        ));

        // If this node has children and is expanded, process them
        if node_ui.expanded && !node_ui.children.is_empty() {
            // Recursively process children
            let children = collect_visible_nodes(
                _plan,
                tree_ui,
                &node_ui.children,
                level + 1,
                selected_node,
                _scroll_offset,
            );

            // Add the children to the result
            result.extend(children);
        }
    }

    result
}

/// Renders the application UI to the terminal
///
/// # Arguments
/// * `f` - The frame to render to
/// * `app` - The application state to render
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // Status bar
                Constraint::Min(3),    // Query input
                Constraint::Min(10),   // Plan output
            ]
            .as_ref(),
        )
        .split(f.size());

    // Draw status bar
    let status_bar = match app.input_mode {
        InputMode::Query => " MODE: Query (Press Enter to execute, Tab to switch to Plan mode, q to quit) ",
        InputMode::Plan => " MODE: Plan (Use arrow keys to navigate, Enter to expand/collapse, Tab to switch to Query mode, q to quit) ",
    };

    let status_bar = Paragraph::new(status_bar)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .block(Block::default());
    f.render_widget(status_bar, chunks[0]);

    // Draw query input
    let query_block = Block::default()
        .title("SQL Query")
        .borders(Borders::ALL)
        .style(match app.input_mode {
            InputMode::Query => Style::default().fg(Color::Yellow),
            InputMode::Plan => Style::default().fg(Color::Gray),
        });

    let query_input = Paragraph::new(app.query.as_str())
        .block(query_block)
        .wrap(Wrap { trim: true });

    f.render_widget(query_input, chunks[1]);

    // Draw plan output
    let plan_block = Block::default()
        .title("Execution Plan")
        .borders(Borders::ALL)
        .style(match app.input_mode {
            InputMode::Query => Style::default().fg(Color::Gray),
            InputMode::Plan => Style::default().fg(Color::Yellow),
        });

    // Get the plan area
    let plan_area = plan_block.inner(chunks[2]);
    f.render_widget(plan_block, chunks[2]);

    // If we have a plan, render it
    if let Some(plan) = &app.plan {
        let exec_plan = match serde_json::from_value::<ExecutionPlan>(plan.clone()) {
            Ok(plan) => Some(plan),
            Err(e) => {
                eprintln!("Error parsing execution plan: {}", e);
                None
            }
        };

        if let Some(exec_plan) = &exec_plan {
            // Check if there was an error in the plan
            if let Some(error) = plan.get("error").and_then(|e| e.as_str()) {
                let error_msg = Paragraph::new(format!("Error executing query:\n{}", error))
                    .style(Style::default().fg(Color::Red))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Error")
                            .border_style(Style::default().fg(Color::Red)),
                    );
                f.render_widget(error_msg, plan_area);
                return;
            }
            // Clear existing tree
            app.plan_tree = PlanTree::default();

            // Build new tree with the root node
            build_plan_tree_ui(&exec_plan.root, &mut app.plan_tree, 0, None);

            // Store hash of the current plan to detect changes
            app.plan_tree.last_plan_hash = Some(plan_hash(plan));

            // Select the first node by default if we have any nodes
            if !app.plan_tree.root_indices.is_empty() {
                let first_node = app.plan_tree.root_indices[0];
                app.selected_node = Some(first_node);
            }

            // Collect all visible nodes
            let visible_nodes = collect_visible_nodes(
                &exec_plan,
                &app.plan_tree,
                &app.plan_tree.root_indices,
                0,
                app.selected_node,
                app.scroll_offset,
            );

            let items: Vec<ListItem> = visible_nodes
                .into_iter()
                .map(|(_, node, is_selected, is_expanded, level, has_children)| {
                    let text =
                        render_plan_node(&node, is_selected, is_expanded, has_children, level);
                    ListItem::new(text)
                })
                .collect();

            // Create and render the list
            let list = List::new(items)
                .block(Block::default())
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            let mut state = ListState::default();
            state.select(app.selected_node);
            f.render_stateful_widget(list, plan_area, &mut state);
        } else {
            // No valid execution plan to display
            let no_plan = Paragraph::new("No valid execution plan to display")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(no_plan, plan_area);
        }
    } else {
        // No plan available
        let no_plan = Paragraph::new("No execution plan available. \n\n")
            .block(Block::default())
            .wrap(Wrap { trim: true });

        f.render_widget(no_plan, plan_area);
    }
}
