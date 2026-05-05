use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
};

use crate::{
    app::{PaneNode, SIDEBAR_HEADER_HEIGHT, SIDEBAR_WIDTH, SplitPath, WhichChild, WindowPage},
    session::SplitDirection,
};

pub(crate) struct RootAreas {
    pub(crate) sidebar: Rect,
    pub(crate) projects_inner: Rect,
    /// Active workspace area (window bar + pane stack), matching `draw_workspace`.
    pub(crate) workspace: Rect,
    /// Top 1-row bar showing window tabs.
    pub(crate) window_tab_bar: Rect,
    pub(crate) pane_stack: Rect,
    pub(crate) status: Rect,
}

pub(crate) fn compute_root_areas(area: Rect) -> RootAreas {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(1)])
        .split(vertical[0]);

    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(SIDEBAR_HEADER_HEIGHT),
            Constraint::Min(1),
        ])
        .split(body[0]);

    let workspace_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(body[1]);

    let projects_block = Block::default()
        .title(" projects ")
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let projects_inner = projects_block.inner(sidebar_chunks[1]);

    RootAreas {
        sidebar: body[0],
        projects_inner,
        workspace: body[1],
        window_tab_bar: workspace_chunks[0],
        pane_stack: workspace_chunks[1],
        status: vertical[1],
    }
}

pub(crate) fn pointer_in_rect(area: Rect, col: u16, row: u16) -> bool {
    col >= area.x
        && col < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

/// Row index into `projects` when the pointer is inside the project list inner area.
pub(crate) fn hit_project_row(
    projects_inner: Rect,
    col: u16,
    row: u16,
    project_count: usize,
) -> Option<usize> {
    if project_count == 0 || !pointer_in_rect(projects_inner, col, row) {
        return None;
    }
    let dy = row.saturating_sub(projects_inner.y) as usize;
    (dy < project_count).then_some(dy)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneHit {
    Title(usize),
    Terminal {
        pane: usize,
        local_col: u16,
        local_row: u16,
    },
}

/// Result of a separator hit-test.
pub(crate) struct SeparatorHit {
    /// Path from the root PaneNode to the Split node whose separator was hit.
    pub(crate) path: SplitPath,
    pub(crate) direction: SplitDirection,
    /// Area of the Split node (used to compute drag ratio).
    pub(crate) area: Rect,
}

// ── Layout helpers ────────────────────────────────────────────────────────────

/// Compute the width (or height) of the first child given the available space
/// and a split ratio in 0.0–1.0. The result is clamped so both halves are ≥ 1.
pub(crate) fn split_first_size(available: u16, ratio: f64) -> u16 {
    if available == 0 {
        return 0;
    }
    let first = (available as f64 * ratio).round() as u16;
    first.max(1).min(available.saturating_sub(1))
}

/// Return (first_rect, sep_rect, second_rect) for a Split node.
pub(crate) fn split_chunks(
    area: Rect,
    direction: SplitDirection,
    ratio: f64,
) -> (Rect, Rect, Rect) {
    match direction {
        SplitDirection::Vertical => {
            let separator = u16::from(area.width > 2);
            let available = area.width.saturating_sub(separator);
            let first_width = split_first_size(available, ratio);
            let second_width = available.saturating_sub(first_width).max(1);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(first_width),
                    Constraint::Length(separator),
                    Constraint::Length(second_width),
                ])
                .split(area);
            (chunks[0], chunks[1], chunks[2])
        }
        SplitDirection::Horizontal => {
            let separator = u16::from(area.height > 2);
            let available = area.height.saturating_sub(separator);
            let first_height = split_first_size(available, ratio);
            let second_height = available.saturating_sub(first_height).max(1);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(first_height),
                    Constraint::Length(separator),
                    Constraint::Length(second_height),
                ])
                .split(area);
            (chunks[0], chunks[1], chunks[2])
        }
    }
}

// ── Hit-testing ───────────────────────────────────────────────────────────────

pub(crate) fn hit_test_pane_stack(
    window: &WindowPage,
    pane_stack: Rect,
    col: u16,
    row: u16,
) -> Option<PaneHit> {
    if window.panes.is_empty() || !pointer_in_rect(pane_stack, col, row) {
        return None;
    }
    hit_pane_node(window, &window.layout, pane_stack, col, row)
}

fn hit_pane_node(
    window: &WindowPage,
    node: &PaneNode,
    area: Rect,
    col: u16,
    row: u16,
) -> Option<PaneHit> {
    match node {
        PaneNode::Leaf(index) => {
            window.panes.get(*index)?;
            hit_pane_leaf(*index, area, col, row)
        }
        PaneNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let (first_chunk, _, second_chunk) = split_chunks(area, *direction, *ratio);
            hit_pane_node(window, first, first_chunk, col, row)
                .or_else(|| hit_pane_node(window, second, second_chunk, col, row))
        }
    }
}

fn hit_pane_leaf(pane_index: usize, area: Rect, col: u16, row: u16) -> Option<PaneHit> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    let title_rect = chunks[0];
    let term_rect = chunks[1];

    if pointer_in_rect(title_rect, col, row) {
        return Some(PaneHit::Title(pane_index));
    }
    if pointer_in_rect(term_rect, col, row) {
        let local_col = col.saturating_sub(term_rect.x);
        let local_row = row.saturating_sub(term_rect.y);
        return Some(PaneHit::Terminal {
            pane: pane_index,
            local_col,
            local_row,
        });
    }
    None
}

pub(crate) fn pane_terminal_rect(
    window: &WindowPage,
    node: &PaneNode,
    area: Rect,
    target_pane: usize,
) -> Option<Rect> {
    match node {
        PaneNode::Leaf(index) => {
            if *index != target_pane || window.panes.get(*index).is_none() {
                return None;
            }
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(area);
            Some(chunks[1])
        }
        PaneNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let (first_chunk, _, second_chunk) = split_chunks(area, *direction, *ratio);
            pane_terminal_rect(window, first, first_chunk, target_pane)
                .or_else(|| pane_terminal_rect(window, second, second_chunk, target_pane))
        }
    }
}

/// Return the separator hit (if any) at (col, row) within pane_stack.
pub(crate) fn hit_separator(
    node: &PaneNode,
    pane_stack: Rect,
    col: u16,
    row: u16,
) -> Option<SeparatorHit> {
    if !pointer_in_rect(pane_stack, col, row) {
        return None;
    }
    let mut path: SplitPath = Vec::new();
    hit_separator_inner(node, pane_stack, col, row, &mut path)
}

fn hit_separator_inner(
    node: &PaneNode,
    area: Rect,
    col: u16,
    row: u16,
    path: &mut SplitPath,
) -> Option<SeparatorHit> {
    match node {
        PaneNode::Leaf(_) => None,
        PaneNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let (first_chunk, sep_chunk, second_chunk) = split_chunks(area, *direction, *ratio);

            // Check if the pointer is on the separator itself.
            if sep_chunk.width > 0 && sep_chunk.height > 0 && pointer_in_rect(sep_chunk, col, row) {
                return Some(SeparatorHit {
                    path: path.clone(),
                    direction: *direction,
                    area,
                });
            }

            // Recurse into first child.
            path.push(WhichChild::First);
            if let Some(hit) = hit_separator_inner(first, first_chunk, col, row, path) {
                return Some(hit);
            }
            path.pop();

            // Recurse into second child.
            path.push(WhichChild::Second);
            if let Some(hit) = hit_separator_inner(second, second_chunk, col, row, path) {
                return Some(hit);
            }
            path.pop();

            None
        }
    }
}
