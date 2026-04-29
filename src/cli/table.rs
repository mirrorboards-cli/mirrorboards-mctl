//! Table rendering with ratatui.

use crossterm::terminal;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};
use std::io::{self, stdout, IsTerminal, Write};

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

    fn text(&self) -> &str {
        match self {
            CellStyle::Normal(s)
            | CellStyle::Success(s)
            | CellStyle::Warning(s)
            | CellStyle::Error(s)
            | CellStyle::Dimmed(s)
            | CellStyle::Highlight(s) => s,
        }
    }
}

/// A row of styled cells
pub struct TableRow {
    cells: Vec<CellStyle>,
    hyperlinks: Vec<Option<String>>,
}

impl TableRow {
    pub fn new(cells: Vec<CellStyle>) -> Self {
        let hyperlinks = vec![None; cells.len()];
        Self { cells, hyperlinks }
    }

    pub fn with_hyperlinks(mut self, hyperlinks: Vec<Option<String>>) -> Self {
        if hyperlinks.len() == self.cells.len() {
            self.hyperlinks = hyperlinks;
        }
        self
    }

    /// Calculate the height needed for this row based on newlines in cells
    fn height(&self) -> u16 {
        self.cells
            .iter()
            .map(|c| c.text().lines().count().max(1) as u16)
            .max()
            .unwrap_or(1)
    }
}

#[derive(Clone)]
struct HyperlinkSpan {
    start_x: u16,
    end_x: u16,
    uri: String,
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
    let (term_width, _) = terminal::size().unwrap_or((80, 24));
    let enable_hyperlinks = io::stdout().is_terminal();

    let header_cells: Vec<Cell> = config
        .headers
        .iter()
        .map(|h| Cell::from(h.as_str()).style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)))
        .collect();
    let header = Row::new(header_cells).height(1);

    let data_rows: Vec<Row> = rows
        .iter()
        .map(|row| {
            let cells: Vec<Cell> = row.cells.iter().map(|c| c.to_cell()).collect();
            Row::new(cells).height(row.height())
        })
        .collect();

    let table = if let Some(title) = &config.title {
        Table::new(data_rows, &config.column_widths)
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
        Table::new(data_rows, &config.column_widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
    };

    let total_row_height: u16 = rows.iter().map(|r| r.height()).sum();
    let height = (total_row_height + 3).max(3);
    let width = term_width.max(1);

    let mut buffer = ratatui::buffer::Buffer::empty(Rect::new(0, 0, width, height));
    table.render(Rect::new(0, 0, width, height), &mut buffer);

    let hyperlink_spans = if enable_hyperlinks {
        build_hyperlink_spans(config, rows, width, height)
    } else {
        vec![Vec::new(); height as usize]
    };

    let mut out = stdout();
    for y in 0..height {
        let line = render_buffer_line(
            &buffer,
            y,
            width,
            hyperlink_spans.get(y as usize).map(Vec::as_slice).unwrap_or(&[]),
        );
        writeln!(out, "{}", line)?;
    }

    out.flush()?;
    Ok(())
}

fn build_hyperlink_spans(
    config: &TableConfig,
    rows: &[TableRow],
    width: u16,
    height: u16,
) -> Vec<Vec<HyperlinkSpan>> {
    let mut spans_by_line = vec![Vec::new(); height as usize];
    if width < 3 || height < 3 {
        return spans_by_line;
    }

    let inner_area = Rect::new(1, 1, width.saturating_sub(2), 1);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(config.column_widths.iter().cloned())
        .split(inner_area);

    let mut row_y = 2u16;
    let max_content_y = height.saturating_sub(1);

    for row in rows {
        let row_height = row.height();

        for (index, maybe_uri) in row.hyperlinks.iter().enumerate() {
            let Some(uri) = maybe_uri else {
                continue;
            };
            let Some(column) = columns.get(index) else {
                continue;
            };
            let Some(cell) = row.cells.get(index) else {
                continue;
            };

            for (line_idx, line) in cell.text().lines().enumerate() {
                let y = row_y + line_idx as u16;
                if y >= max_content_y {
                    break;
                }

                let visible_width = line.chars().count().min(column.width as usize) as u16;
                if visible_width == 0 {
                    continue;
                }

                spans_by_line[y as usize].push(HyperlinkSpan {
                    start_x: column.x,
                    end_x: column.x + visible_width,
                    uri: uri.clone(),
                });
            }
        }

        row_y = row_y.saturating_add(row_height);
        if row_y >= max_content_y {
            break;
        }
    }

    spans_by_line
}

fn render_buffer_line(
    buffer: &ratatui::buffer::Buffer,
    y: u16,
    width: u16,
    hyperlink_spans: &[HyperlinkSpan],
) -> String {
    let mut line = String::new();
    let mut current_style = Style::default();
    let mut current_link: Option<&str> = None;

    for x in 0..width {
        let cell = buffer.cell((x, y)).expect("buffer cell should exist");
        let style = cell.style();
        let next_link = hyperlink_spans
            .iter()
            .find(|span| x >= span.start_x && x < span.end_x)
            .map(|span| span.uri.as_str());

        if next_link != current_link {
            if current_link.is_some() {
                line.push_str(&osc8_close());
            }
            if let Some(uri) = next_link {
                line.push_str(&osc8_open(uri));
            }
            current_link = next_link;
        }

        if style != current_style {
            if current_style != Style::default() {
                line.push_str("\x1b[0m");
            }
            if style != Style::default() {
                line.push_str(&ansi_prefix(&style));
            }
            current_style = style;
        }

        line.push_str(cell.symbol());
    }

    if current_style != Style::default() {
        line.push_str("\x1b[0m");
    }
    if current_link.is_some() {
        line.push_str(&osc8_close());
    }

    line
}

fn ansi_prefix(style: &Style) -> String {
    let mut codes = Vec::new();

    if style.add_modifier.contains(Modifier::BOLD) {
        codes.push("1");
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
        codes.push(code);
    }

    if codes.is_empty() {
        String::new()
    } else {
        format!("\x1b[{}m", codes.join(";"))
    }
}

fn osc8_open(uri: &str) -> String {
    format!("\x1b]8;;{}\x1b\\", uri)
}

fn osc8_close() -> String {
    "\x1b]8;;\x1b\\".to_string()
}

#[cfg(test)]
mod tests {
    use super::{build_hyperlink_spans, osc8_close, osc8_open, CellStyle, TableConfig, TableRow};
    use ratatui::layout::Constraint;

    #[test]
    fn hyperlink_spans_follow_first_data_column() {
        let config = TableConfig::new(vec!["Path", "Branch"])
            .with_widths(vec![Constraint::Length(10), Constraint::Length(8)]);
        let rows = vec![
            TableRow::new(vec![CellStyle::highlight("repo/path"), CellStyle::normal("main")])
                .with_hyperlinks(vec![Some("file:///tmp/repo".to_string()), None]),
        ];

        let spans = build_hyperlink_spans(&config, &rows, 24, 4);
        let line_spans = &spans[2];

        assert_eq!(line_spans.len(), 1);
        assert_eq!(line_spans[0].start_x, 1);
        assert_eq!(line_spans[0].end_x, 10);
        assert_eq!(line_spans[0].uri, "file:///tmp/repo");
    }

    #[test]
    fn osc8_sequences_are_well_formed() {
        let open = osc8_open("file:///tmp/repo");
        let close = osc8_close();

        assert_eq!(open, "\x1b]8;;file:///tmp/repo\x1b\\");
        assert_eq!(close, "\x1b]8;;\x1b\\");
    }
}
