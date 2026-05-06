use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use vt100::{Cell, Screen};

use crate::{
    app::{App, PaneNode, RenameState, SIDEBAR_HEADER_HEIGHT, WhichChild, WindowPage},
    layout,
    session::SplitDirection,
    terminal::{TerminalStatus, TerminalTab},
};

pub(crate) fn draw(frame: &mut Frame<'_>, app: &App) {
    let areas = layout::compute_root_areas(frame.area());

    draw_projects(frame, app, areas.sidebar);
    draw_workspace(frame, app, areas.workspace);
    draw_status(frame, app, areas.status);

    if app.show_help {
        draw_help(frame, centered_rect(72, 68, frame.area()));
    }
}

fn draw_projects(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let sidebar_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(SIDEBAR_HEADER_HEIGHT),
            Constraint::Min(1),
        ])
        .split(area);

    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            " kandume",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!(" v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(header, sidebar_vertical[0]);

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
            let session_count = project.windows.len();
            ListItem::new(Line::from(format!(
                "{marker} {} {}  {} sessions",
                index + 1,
                project.name,
                session_count
            )))
            .style(style)
        })
        .collect();

    let projects = List::new(items).block(
        Block::default()
            .title(" projects ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(projects, sidebar_vertical[1]);
}

fn draw_workspace(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    draw_windows(frame, app, chunks[0]);

    if let Some(window) = app.active_window() {
        draw_panes(frame, window, app.separator_highlight(), chunks[1]);
    } else {
        frame.render_widget(Paragraph::new("no session"), chunks[1]);
    }
}

fn draw_windows(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let Some(project) = app.active_project() else {
        frame.render_widget(Paragraph::new("no project"), area);
        return;
    };

    let spans = project
        .windows
        .iter()
        .enumerate()
        .flat_map(|(index, window)| {
            let style = if index == project.active_window {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            [
                Span::styled(format!(" {}:{} ", index + 1, window.name), style),
                Span::raw(" "),
            ]
        })
        .collect::<Vec<_>>();

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_panes(
    frame: &mut Frame<'_>,
    window: &WindowPage,
    hover_sep: Option<&[WhichChild]>,
    area: Rect,
) {
    if window.panes.is_empty() {
        frame.render_widget(Paragraph::new("no panes"), area);
        return;
    }

    draw_pane_node(frame, window, &window.layout, hover_sep, area);
}

fn draw_pane_node(
    frame: &mut Frame<'_>,
    window: &WindowPage,
    node: &PaneNode,
    hover_sep: Option<&[WhichChild]>,
    area: Rect,
) {
    match node {
        PaneNode::Leaf(index) => {
            if let Some(pane) = window.panes.get(*index) {
                let is_active = *index == window.active_pane;
                draw_pane_leaf(frame, pane, is_active, area);
                if is_active {
                    frame.render_widget(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                        area,
                    );
                }
            }
        }
        PaneNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let (first_chunk, sep_chunk, second_chunk) =
                layout::split_chunks(area, *direction, *ratio);

            // Highlight the separator if `hover_sep` points exactly to this node.
            let highlighted = hover_sep == Some(&[]);

            // Narrow `hover_sep` for each child.
            let first_hover: Option<&[WhichChild]> = match hover_sep {
                Some([WhichChild::First, rest @ ..]) => Some(rest),
                _ => None,
            };
            let second_hover: Option<&[WhichChild]> = match hover_sep {
                Some([WhichChild::Second, rest @ ..]) => Some(rest),
                _ => None,
            };

            let sep_color = if highlighted {
                Color::Yellow
            } else {
                Color::DarkGray
            };

            match direction {
                SplitDirection::Vertical => {
                    draw_pane_node(frame, window, first, first_hover, first_chunk);
                    if sep_chunk.width > 0 {
                        frame.render_widget(
                            Block::default()
                                .borders(Borders::LEFT)
                                .border_style(Style::default().fg(sep_color)),
                            sep_chunk,
                        );
                    }
                    draw_pane_node(frame, window, second, second_hover, second_chunk);
                }
                SplitDirection::Horizontal => {
                    draw_pane_node(frame, window, first, first_hover, first_chunk);
                    if sep_chunk.height > 0 {
                        frame.render_widget(
                            Block::default()
                                .borders(Borders::TOP)
                                .border_style(Style::default().fg(sep_color)),
                            sep_chunk,
                        );
                    }
                    draw_pane_node(frame, window, second, second_hover, second_chunk);
                }
            }
        }
    }
}

fn draw_pane_leaf(frame: &mut Frame<'_>, pane: &TerminalTab, active: bool, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);
    frame.render_widget(Paragraph::new(pane_title(pane, active)), chunks[0]);
    let screen = pane.screen();
    frame.render_widget(Paragraph::new(terminal_lines(screen)), chunks[1]);
    if active && !screen.hide_cursor() {
        let (row, col) = screen.cursor_position();
        let content = chunks[1];
        let x = content
            .x
            .saturating_add(col)
            .min(content.x + content.width.saturating_sub(1));
        let y = content
            .y
            .saturating_add(row)
            .min(content.y + content.height.saturating_sub(1));
        frame.set_cursor_position((x, y));
    }
}

fn pane_title(pane: &TerminalTab, active: bool) -> Line<'static> {
    let status = pane.status();
    let status_style = match status {
        TerminalStatus::Running => Style::default().fg(Color::DarkGray),
        TerminalStatus::Active => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        TerminalStatus::Completed => Style::default().fg(Color::Green),
        TerminalStatus::Failed => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    };
    let marker = match status {
        TerminalStatus::Running => "-",
        TerminalStatus::Active => "*",
        TerminalStatus::Completed => "ok",
        TerminalStatus::Failed => "!",
    };
    let title_style = if active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    Line::from(vec![
        Span::styled(format!(" {marker} "), status_style),
        Span::styled(pane.name.clone(), title_style),
    ])
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

fn draw_status(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let line = match &app.rename_state {
        RenameState::Project { buffer } => rename_line("rename project: ", buffer),
        RenameState::Window { buffer } => rename_line("rename session: ", buffer),
        RenameState::Pane { buffer } => rename_line("rename pane: ", buffer),
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
                    " t:project c:session %/\":split n/p:project [/]:session o/;:pane x:close-pane ,/./r:rename d:save+quit ?:help",
                ),
            ])
        }
    };

    frame.render_widget(Paragraph::new(line), area);
}

fn rename_line(label: &'static str, buffer: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Yellow)),
        Span::raw(buffer.to_string()),
        Span::styled(
            "  Enter=ok Esc=cancel",
            Style::default().fg(Color::DarkGray),
        ),
    ])
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
        Line::from("Ctrl-b c    new session in current project"),
        Line::from("Ctrl-b %    split current session vertically"),
        Line::from("Ctrl-b \"    split current session horizontally"),
        Line::from("Ctrl-b n    next project"),
        Line::from("Ctrl-b p    previous project"),
        Line::from("Ctrl-b 1-9  select project"),
        Line::from("Ctrl-b ]    next session"),
        Line::from("Ctrl-b [    previous session"),
        Line::from("Ctrl-b o    next pane"),
        Line::from("Ctrl-b ;    previous pane"),
        Line::from(
            "Ctrl-b x    close pane (last pane closes session → prior session; lone session → home project)",
        ),
        Line::from("Ctrl-b ,    rename project"),
        Line::from("Ctrl-b .    rename session"),
        Line::from("Ctrl-b r    rename pane"),
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
