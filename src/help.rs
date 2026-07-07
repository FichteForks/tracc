use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

const BINDINGS: &[(&str, &str)] = &[
    ("j / k", "move selection down / up"),
    ("J / K", "go to next / previous day"),
    ("gg / G", "jump to first / last item"),
    ("gt", "go to today"),
    ("gd", "load a specific day"),
    ("y / p", "yank / paste current item"),
    ("o", "insert new item"),
    ("a / i", "edit item text (append / insert)"),
    ("A / I", "edit item time (append / insert)"),
    ("r / R", "replace item text / time"),
    ("d", "delete current item"),
    ("- / +", "shift time to previous / next 5-minute mark"),
    ("u / Ctrl+r", "undo / redo"),
    ("q", "quit"),
    ("?", "toggle this help"),
];

pub fn area(frame_area: Rect) -> Rect {
    let key_width = BINDINGS.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
    let content_width = BINDINGS
        .iter()
        .map(|(_, desc)| key_width + 2 + desc.len())
        .max()
        .unwrap_or(20);
    let width = (content_width as u16 + 2)
        .min(frame_area.width.saturating_sub(2))
        .max(20);
    let height = (BINDINGS.len() as u16 + 2).min(frame_area.height.saturating_sub(2));
    let x = frame_area.x + (frame_area.width.saturating_sub(width)) / 2;
    let y = frame_area.y + (frame_area.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width, height)
}

pub fn render(frame: &mut ratatui::Frame<'_>, area: Rect) {
    let key_width = BINDINGS.iter().map(|(key, _)| key.len()).max().unwrap_or(0);
    let lines: Vec<Line> = BINDINGS
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("{key:<key_width$}  "),
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(*desc),
            ])
        })
        .collect();
    let block = Block::default()
        .title(" keybindings ")
        .borders(Borders::ALL);

    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
