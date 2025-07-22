use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct App {
    pub should_quit: bool,
    pub query: String,
    pub plan: Option<serde_json::Value>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            query: String::new(),
            plan: None,
        }
    }

    pub fn on_tick(&mut self) {
        // Update app state on each tick if needed
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => self.should_quit = true,
            _ => {}
        }
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(20), // Query input
                Constraint::Percentage(80), // Plan output
            ]
            .as_ref(),
        )
        .split(f.size());

    // Draw query input
    let query_block = Block::default()
        .title("SQL Query")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let query_paragraph = Paragraph::new(app.query.as_str())
        .block(query_block)
        .wrap(Wrap { trim: true });
    f.render_widget(query_paragraph, chunks[0]);

    // Draw execution plan
    let plan_block = Block::default()
        .title("Execution Plan")
        .borders(Borders::ALL);

    let plan_text = match &app.plan {
        Some(plan) => format!("{:#?}", plan), // Simple debug output for now
        None => "No execution plan available. Run a query first.".to_string(),
    };

    let plan_paragraph = Paragraph::new(plan_text)
        .block(plan_block)
        .wrap(Wrap { trim: true });
    f.render_widget(plan_paragraph, chunks[1]);
}
