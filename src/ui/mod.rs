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
fn build_plan_tree_ui(
    node: &PlanNode,
    tree_ui: &mut PlanTree,
    node_index: usize,
    parent_index: Option<usize>,
) {
    let children_count = node.plans.len();
    let mut child_indices = Vec::with_capacity(children_count);

    // Add children
    for (i, child) in node.plans.iter().enumerate() {
        let child_index = tree_ui.nodes.len();
        child_indices.push(child_index);

        tree_ui.nodes.push(PlanNodeUI {
            expanded: i == 0, // Expand first child by default
            children: Vec::new(),
        });

        build_plan_tree_ui(child, tree_ui, child_index, Some(node_index));
    }

    // Update current node with children indices
    if let Some(parent_idx) = parent_index {
        tree_ui.nodes[parent_idx].children = child_indices;
    } else {
        tree_ui.root_indices = child_indices;
    }
}

#[derive(Debug, Default)]
pub struct PlanNodeUI {
    pub expanded: bool,
    pub children: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct PlanTree {
    pub nodes: Vec<PlanNodeUI>,
    pub root_indices: Vec<usize>,
}

pub struct App {
    pub should_quit: bool,
    pub query: String,
    pub plan: Option<serde_json::Value>,
    pub plan_tree: PlanTree,
    pub selected_node: Option<usize>,
    pub scroll_offset: u16,
    pub input_mode: InputMode,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputMode {
    Query,
    Plan,
}

impl App {
    pub fn new() -> Self {
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

    pub fn on_tick(&mut self) {
        // Update app state on each tick if needed
    }

    pub fn on_key(&mut self, key: KeyCode) -> bool {
        match self.input_mode {
            InputMode::Query => self.handle_query_mode(key),
            InputMode::Plan => self.handle_plan_mode(key),
        }
    }

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
                // Execute the query and get the plan
                // This will be handled by the main application
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
                self.move_selection(1);
                true
            }
            KeyCode::Up => {
                self.move_selection(-1);
                true
            }
            KeyCode::Right => {
                self.expand_node();
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

    fn move_selection(&mut self, delta: i32) {
        if self.plan_tree.nodes.is_empty() {
            return;
        }

        let node_count = self.plan_tree.nodes.len();
        let current = self.selected_node.unwrap_or(0) as i32;
        let new_selection = (current + delta).max(0).min(node_count as i32 - 1) as usize;

        if new_selection != current as usize {
            self.selected_node = Some(new_selection);
            // TODO: Update scroll position if needed
        }
    }

    fn expand_node(&mut self) {
        if let Some(node_idx) = self.selected_node {
            if node_idx < self.plan_tree.nodes.len() {
                self.plan_tree.nodes[node_idx].expanded = true;
            }
        }
    }

    fn collapse_node(&mut self) {
        if let Some(node_idx) = self.selected_node {
            if node_idx < self.plan_tree.nodes.len() {
                self.plan_tree.nodes[node_idx].expanded = false;
            }
        }
    }

    fn toggle_node(&mut self) {
        if let Some(node_idx) = self.selected_node {
            if node_idx < self.plan_tree.nodes.len() {
                let node = &mut self.plan_tree.nodes[node_idx];
                node.expanded = !node.expanded;
            }
        }
    }
}

/// Collects all visible nodes in the plan tree for rendering
fn collect_visible_nodes(
    plan: &ExecutionPlan,
    tree_ui: &PlanTree,
    node_indices: &[usize],
    level: usize,
    selected_node: Option<usize>,
    scroll_offset: u16,
) -> Vec<(usize, PlanNode, bool, bool, usize, bool)> {
    let mut result = Vec::new();

    for &node_idx in node_indices {
        let node_ui = &tree_ui.nodes[node_idx];
        let is_selected = Some(node_idx) == selected_node;

        // Add the node itself
        result.push((
            node_idx,
            plan.root.clone(), // We'll need to get the actual node here
            is_selected,
            node_ui.expanded,
            level,
            !node_ui.children.is_empty(),
        ));

        // Add children if expanded
        if node_ui.expanded && !node_ui.children.is_empty() {
            let children = collect_visible_nodes(
                plan,
                tree_ui,
                &node_ui.children,
                level + 1,
                selected_node,
                scroll_offset,
            );
            result.extend(children);
        }
    }

    result
}

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
    if let (Some(plan), Some(exec_plan)) = (
        &app.plan,
        &app.plan.as_ref().and_then(|p| {
            let parsed = serde_json::from_value::<ExecutionPlan>(p.clone());
            if let Err(e) = &parsed {
                println!("DEBUG - Error parsing plan: {}", e);
            }
            parsed.ok()
        }),
    ) {
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
        // Build the plan tree UI if not already done
        if app.plan_tree.nodes.is_empty() {
            build_plan_tree_ui(&exec_plan.root, &mut app.plan_tree, 0, None);
            // Select the first node by default
            if !app.plan_tree.root_indices.is_empty() {
                app.selected_node = Some(app.plan_tree.root_indices[0]);
            }
        }

        // Collect all visible nodes
        let visible_nodes = collect_visible_nodes(
            exec_plan,
            &app.plan_tree,
            &app.plan_tree.root_indices,
            0,
            app.selected_node,
            app.scroll_offset,
        );

        // Create a list of rendered nodes
        let items: Vec<ListItem> = visible_nodes
            .into_iter()
            .map(|(_, node, is_selected, is_expanded, level, has_children)| {
                let text = render_plan_node(&node, is_selected, is_expanded, has_children, level);
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
        // No plan available
        let no_plan = Paragraph::new("No execution plan available. \n\n")
            .block(Block::default())
            .wrap(Wrap { trim: true });

        f.render_widget(no_plan, plan_area);
    }
}
