use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use vt100::{Cell, Screen};

use crate::app::{App, RenameState, SIDEBAR_WIDTH};

pub(crate) fn draw(frame: &mut Frame<'_>, app: &App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(1)])
        .split(vertical[0]);

    draw_tabs(frame, app, body[0]);
    draw_terminal(frame, app, body[1]);
    draw_status(frame, app, vertical[1]);

    if app.show_help {
        draw_help(frame, centered_rect(68, 56, frame.area()));
    }
}

fn draw_tabs(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let items: Vec<ListItem<'_>> = app
        .projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            let marker = if index == app.active_project {
                ">"
            } else {
                " "
            };
            let style = if index == app.active_project {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Line::from(format!(
                "{marker} {} {} ({})",
                index + 1,
                project.name,
                project.sessions.len()
            )))
            .style(style)
        })
        .collect();

    let tabs = List::new(items).block(
        Block::default()
            .title(" projects ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(tabs, area);
}

fn draw_terminal(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    draw_sessions(frame, app, chunks[0]);

    let terminal = app.active_terminal().map_or_else(
        || Paragraph::new("no terminal"),
        |tab| Paragraph::new(terminal_lines(tab.screen())),
    );
    frame.render_widget(terminal, chunks[1]);
}

fn terminal_lines(screen: &Screen) -> Vec<Line<'static>> {
    let (rows, cols) = screen.size();
    (0..rows)
        .map(|row| {
            let spans = (0..cols)
                .filter_map(|col| screen.cell(row, col))
                .filter(|cell| !cell.is_wide_continuation())
                .map(cell_span)
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn cell_span(cell: &Cell) -> Span<'static> {
    let content = if cell.has_contents() {
        cell.contents()
    } else {
        " ".to_string()
    };
    Span::styled(content, cell_style(cell))
}

fn cell_style(cell: &Cell) -> Style {
    let mut style = Style::default();

    let mut fg = cell.fgcolor();
    let mut bg = cell.bgcolor();
    if cell.inverse() {
        std::mem::swap(&mut fg, &mut bg);
    }

    if let Some(color) = terminal_color(fg) {
        style = style.fg(color);
    }
    if let Some(color) = terminal_color(bg) {
        style = style.bg(color);
    }
    if cell.bold() {
        style = style.add_modifier(Modifier::BOLD);
    }
    if cell.italic() {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if cell.underline() {
        style = style.add_modifier(Modifier::UNDERLINED);
    }

    style
}

fn terminal_color(color: vt100::Color) -> Option<Color> {
    match color {
        vt100::Color::Default => None,
        vt100::Color::Idx(index) => Some(Color::Indexed(index)),
        vt100::Color::Rgb(red, green, blue) => Some(Color::Rgb(red, green, blue)),
    }
}

fn draw_sessions(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(project) = app.active_project() else {
        frame.render_widget(Paragraph::new("no project"), area);
        return;
    };

    let spans = project
        .sessions
        .iter()
        .enumerate()
        .flat_map(|(index, session)| {
            let style = if index == project.active_session {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            [
                Span::styled(format!(" {}:{} ", index + 1, session.name), style),
                Span::raw(" "),
            ]
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_status(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let line = match &app.rename_state {
        RenameState::Project { buffer } => Line::from(vec![
            Span::styled("rename project: ", Style::default().fg(Color::Yellow)),
            Span::raw(buffer.as_str()),
            Span::styled(
                "  Enter=ok Esc=cancel",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        RenameState::Session { buffer } => Line::from(vec![
            Span::styled("rename session: ", Style::default().fg(Color::Yellow)),
            Span::raw(buffer.as_str()),
            Span::styled(
                "  Enter=ok Esc=cancel",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        RenameState::Idle => {
            let prefix = if app.prefix_active {
                Span::styled(
                    "PREFIX ",
                    Style::default().fg(Color::Black).bg(Color::Yellow),
                )
            } else {
                Span::styled("Ctrl-b", Style::default().fg(Color::Cyan))
            };
            Line::from(vec![
                prefix,
                Span::raw(
                    " t:project c:session n/p:project [/]:session x:close ,:rename-project r:rename-session d:save+quit ?:help",
                ),
            ])
        }
    };

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_help(frame: &mut Frame<'_>, area: Rect) {
    let text = vec![
        Line::from(Span::styled(
            "kandume keys",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Ctrl-b t    new project"),
        Line::from("Ctrl-b c    new terminal session in current project"),
        Line::from("Ctrl-b n    next project"),
        Line::from("Ctrl-b p    previous project"),
        Line::from("Ctrl-b 1-9  select project"),
        Line::from("Ctrl-b ]    next terminal session"),
        Line::from("Ctrl-b [    previous terminal session"),
        Line::from("Ctrl-b x    close current terminal session"),
        Line::from("Ctrl-b ,    rename project"),
        Line::from("Ctrl-b r    rename terminal session"),
        Line::from("Ctrl-b d    save session and quit"),
        Line::from("Ctrl-b q    quit"),
        Line::from("Ctrl-b ?    toggle this help"),
    ];
    let block = Block::default()
        .title(" help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
