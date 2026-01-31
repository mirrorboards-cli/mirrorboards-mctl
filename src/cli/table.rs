//! Table rendering with ratatui.

use crossterm::terminal;
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};
use std::io::{self, stdout, Write};

/// Represents a styled cell value
#[derive(Clone)]
pub enum CellStyle {
    Normal(String),
    Success(String),
    Warning(String),
    Error(String),
    Dimmed(String),
    Highlight(String),
}

impl CellStyle {
    pub fn normal(s: impl Into<String>) -> Self {
        CellStyle::Normal(s.into())
    }

    pub fn success(s: impl Into<String>) -> Self {
        CellStyle::Success(s.into())
    }

    pub fn warning(s: impl Into<String>) -> Self {
        CellStyle::Warning(s.into())
    }

    pub fn error(s: impl Into<String>) -> Self {
        CellStyle::Error(s.into())
    }

    pub fn dimmed(s: impl Into<String>) -> Self {
        CellStyle::Dimmed(s.into())
    }

    pub fn highlight(s: impl Into<String>) -> Self {
        CellStyle::Highlight(s.into())
    }

    fn to_cell(&self) -> Cell<'_> {
        match self {
            CellStyle::Normal(s) => Cell::from(s.as_str()),
            CellStyle::Success(s) => Cell::from(s.as_str()).style(Style::default().fg(Color::Green)),
            CellStyle::Warning(s) => Cell::from(s.as_str()).style(Style::default().fg(Color::Yellow)),
            CellStyle::Error(s) => Cell::from(s.as_str()).style(Style::default().fg(Color::Red)),
            CellStyle::Dimmed(s) => Cell::from(s.as_str()).style(Style::default().fg(Color::DarkGray)),
            CellStyle::Highlight(s) => Cell::from(s.as_str()).style(Style::default().fg(Color::Cyan)),
        }
    }
}

/// A row of styled cells
pub struct TableRow {
    cells: Vec<CellStyle>,
}

impl TableRow {
    pub fn new(cells: Vec<CellStyle>) -> Self {
        Self { cells }
    }
}

/// Configuration for rendering a table
pub struct TableConfig {
    pub title: Option<String>,
    pub headers: Vec<String>,
    pub column_widths: Vec<Constraint>,
}

impl TableConfig {
    pub fn new(headers: Vec<&str>) -> Self {
        let len = headers.len();
        Self {
            title: None,
            headers: headers.into_iter().map(String::from).collect(),
            column_widths: vec![Constraint::Percentage(100 / len as u16); len],
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_widths(mut self, widths: Vec<Constraint>) -> Self {
        self.column_widths = widths;
        self
    }
}

/// Render a table to stdout using ratatui (one-shot, no TUI mode)
pub fn render_table(config: &TableConfig, rows: &[TableRow]) -> io::Result<()> {
    // Get terminal width
    let (term_width, _) = terminal::size().unwrap_or((80, 24));

    // Build header row
    let header_cells: Vec<Cell> = config
        .headers
        .iter()
        .map(|h| {
            Cell::from(h.as_str())
                .style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan))
        })
        .collect();
    let header = Row::new(header_cells).height(1);

    // Build data rows
    let data_rows: Vec<Row> = rows
        .iter()
        .map(|row| {
            let cells: Vec<Cell> = row.cells.iter().map(|c| c.to_cell()).collect();
            Row::new(cells).height(1)
        })
        .collect();

    // Create table widget
    let table = if let Some(title) = &config.title {
        Table::new(data_rows.clone(), &config.column_widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title(title.as_str())
                    .title_style(Style::default().bold().fg(Color::White)),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
    } else {
        Table::new(data_rows.clone(), &config.column_widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
    };

    // Render to a buffer and print - use full terminal width with minimum 80
    let height = rows.len() as u16 + 4; // rows + header + borders
    let width = term_width.max(80);

    // Create a buffer to render into
    let mut buffer = ratatui::buffer::Buffer::empty(ratatui::layout::Rect::new(0, 0, width, height));

    // Render the table widget to the buffer
    table.render(ratatui::layout::Rect::new(0, 0, width, height), &mut buffer);

    // Print buffer contents
    for y in 0..height {
        for x in 0..width {
            let cell = buffer.cell((x, y)).unwrap();

            // Apply style
            let content = cell.symbol();
            let style = cell.style();

            if style.fg.is_some() || style.add_modifier.contains(Modifier::BOLD) {
                print!("{}", apply_ansi_style(content, &style));
            } else {
                print!("{}", content);
            }
        }
        println!();
    }

    stdout().flush()?;
    Ok(())
}

fn apply_ansi_style(content: &str, style: &Style) -> String {
    let mut result = String::new();

    // Start codes
    if style.add_modifier.contains(Modifier::BOLD) {
        result.push_str("\x1b[1m");
    }

    if let Some(fg) = style.fg {
        let code = match fg {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::Gray => "37",
            Color::DarkGray => "90",
            Color::LightRed => "91",
            Color::LightGreen => "92",
            Color::LightYellow => "93",
            Color::LightBlue => "94",
            Color::LightMagenta => "95",
            Color::LightCyan => "96",
            Color::White => "97",
            _ => "39",
        };
        result.push_str(&format!("\x1b[{}m", code));
    }

    result.push_str(content);

    // Reset
    if style.fg.is_some() || style.add_modifier.contains(Modifier::BOLD) {
        result.push_str("\x1b[0m");
    }

    result
}
