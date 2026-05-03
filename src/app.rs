use std::{
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use anyhow::{Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent};

use crate::{
    input::{InputAction, InputState, encode_key},
    session::{self, SessionProject, SessionState, SessionWindow, SplitDirection},
    terminal::{TerminalTab, default_cwd, normalize_cwd},
};

pub(crate) const SIDEBAR_WIDTH: u16 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RenameState {
    Idle,
    Project { buffer: String },
    Window { buffer: String },
    Pane { buffer: String },
}

pub(crate) struct Project {
    pub(crate) name: String,
    pub(crate) cwd: PathBuf,
    pub(crate) windows: Vec<WindowPage>,
    pub(crate) active_window: usize,
}

pub(crate) struct WindowPage {
    pub(crate) name: String,
    pub(crate) panes: Vec<TerminalTab>,
    pub(crate) active_pane: usize,
    pub(crate) split_direction: SplitDirection,
}

pub(crate) struct App {
    pub(crate) projects: Vec<Project>,
    pub(crate) active_project: usize,
    pub(crate) prefix_active: bool,
    pub(crate) show_help: bool,
    pub(crate) rename_state: RenameState,
    input: InputState,
    output_tx: Sender<(u64, Vec<u8>)>,
    output_rx: Receiver<(u64, Vec<u8>)>,
    next_tab_id: u64,
    terminal_cols: u16,
    terminal_rows: u16,
    should_quit: bool,
}

impl App {
    pub(crate) fn new(cols: u16, rows: u16) -> Result<Self> {
        let (output_tx, output_rx) = channel();
        let terminal_cols = cols.saturating_sub(SIDEBAR_WIDTH).max(1);
        let terminal_rows = rows.saturating_sub(3).max(1);

        let mut app = Self {
            projects: Vec::new(),
            active_project: 0,
            prefix_active: false,
            show_help: false,
            rename_state: RenameState::Idle,
            input: InputState::default(),
            output_tx,
            output_rx,
            next_tab_id: 1,
            terminal_cols,
            terminal_rows,
            should_quit: false,
        };

        app.restore_or_create_default()?;
        app.resize_all_panes()?;
        Ok(app)
    }

    pub(crate) fn active_project(&self) -> Option<&Project> {
        self.projects.get(self.active_project)
    }

    pub(crate) fn active_window(&self) -> Option<&WindowPage> {
        self.active_project()
            .and_then(|project| project.windows.get(project.active_window))
    }

    pub(crate) fn active_terminal(&self) -> Option<&TerminalTab> {
        self.active_window()
            .and_then(|window| window.panes.get(window.active_pane))
    }

    pub(crate) fn tick(&mut self) -> Result<bool> {
        self.drain_pty_output();

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => self.handle_key(key)?,
                Event::Resize(cols, rows) => self.resize(cols, rows)?,
                Event::FocusGained | Event::FocusLost | Event::Mouse(_) | Event::Paste(_) => {}
            }
        }

        self.prefix_active = self.input.is_prefix_active();
        Ok(!self.should_quit)
    }

    pub(crate) fn save_session(&self) -> Result<()> {
        let state = SessionState {
            active_project: self.active_project,
            projects: self.projects.iter().map(Project::session_project).collect(),
        };
        session::save(&state)
    }

    fn restore_or_create_default(&mut self) -> Result<()> {
        if let Some(state) = session::load()? {
            for project in state.projects {
                self.restore_project(project)?;
            }

            if !self.projects.is_empty() {
                self.active_project = state.active_project.min(self.projects.len() - 1);
                return Ok(());
            }
        }

        self.new_project()?;
        Ok(())
    }

    fn restore_project(&mut self, project: SessionProject) -> Result<()> {
        let mut windows = Vec::new();
        for window in project.windows {
            windows.push(self.restore_window(&project.cwd, window)?);
        }

        if windows.is_empty() {
            windows.push(self.new_window_for_cwd(project.cwd.clone(), "window 1".to_string())?);
        }

        let active_window = project.active_window.min(windows.len() - 1);
        self.projects.push(Project {
            name: project.name,
            cwd: project.cwd,
            windows,
            active_window,
        });
        Ok(())
    }

    fn restore_window(&mut self, cwd: &Path, window: SessionWindow) -> Result<WindowPage> {
        let mut panes = Vec::new();
        for pane in window.panes {
            let id = self.allocate_tab_id();
            let terminal = TerminalTab::from_session(
                id,
                pane,
                self.terminal_cols,
                self.terminal_rows,
                self.output_tx.clone(),
            )?;
            panes.push(terminal);
        }

        if panes.is_empty() {
            panes.push(self.spawn_pane(cwd.to_path_buf(), "pane 1".to_string())?);
        }

        Ok(WindowPage {
            name: window.name,
            active_pane: window.active_pane.min(panes.len() - 1),
            split_direction: window.split_direction,
            panes,
        })
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.handle_rename_key(key) {
            return Ok(());
        }

        let action = self.input.handle_key(key);
        match action {
            InputAction::Send(bytes) => {
                if !self.show_help {
                    self.write_to_active(&bytes)?;
                }
            }
            InputAction::NewProject => self.new_project()?,
            InputAction::NewWindow => self.new_window()?,
            InputAction::SplitVertical => self.split_pane(SplitDirection::Vertical)?,
            InputAction::SplitHorizontal => self.split_pane(SplitDirection::Horizontal)?,
            InputAction::NextProject => self.next_project(),
            InputAction::PreviousProject => self.previous_project(),
            InputAction::SelectProject(index) => self.select_project(index),
            InputAction::NextWindow => self.next_window(),
            InputAction::PreviousWindow => self.previous_window(),
            InputAction::NextPane => self.next_pane(),
            InputAction::PreviousPane => self.previous_pane(),
            InputAction::ClosePane => self.close_active_pane()?,
            InputAction::StartProjectRename => self.start_project_rename(),
            InputAction::StartWindowRename => self.start_window_rename(),
            InputAction::StartPaneRename => self.start_pane_rename(),
            InputAction::SaveAndQuit => {
                self.save_session()?;
                self.should_quit = true;
            }
            InputAction::Quit => self.should_quit = true,
            InputAction::ToggleHelp => self.show_help = !self.show_help,
            InputAction::None => {}
        }
        Ok(())
    }

    fn handle_rename_key(&mut self, key: KeyEvent) -> bool {
        match &mut self.rename_state {
            RenameState::Idle => false,
            RenameState::Project { buffer } => {
                apply_rename_key(key, buffer, |name| {
                    if let Some(project) = self.projects.get_mut(self.active_project) {
                        project.name = name;
                    }
                })
            }
            RenameState::Window { buffer } => apply_rename_key(key, buffer, |name| {
                if let Some(project) = self.projects.get_mut(self.active_project)
                    && let Some(window) = project.windows.get_mut(project.active_window)
                {
                    window.name = name;
                }
            }),
            RenameState::Pane { buffer } => apply_rename_key(key, buffer, |name| {
                if let Some(project) = self.projects.get_mut(self.active_project)
                    && let Some(window) = project.windows.get_mut(project.active_window)
                    && let Some(pane) = window.panes.get_mut(window.active_pane)
                {
                    pane.name = name;
                }
            }),
        }
    }

    fn new_project(&mut self) -> Result<()> {
        let cwd = default_cwd()?;
        let name = project_name(&cwd);
        let window = self.new_window_for_cwd(cwd.clone(), "window 1".to_string())?;
        self.projects.push(Project {
            name,
            cwd,
            windows: vec![window],
            active_window: 0,
        });
        self.active_project = self.projects.len() - 1;
        self.resize_all_panes()
    }

    fn new_window(&mut self) -> Result<()> {
        let active_project = self.active_project;
        let Some(project) = self.projects.get(active_project) else {
            return self.new_project();
        };
        let cwd = normalize_cwd(&project.cwd);
        let name = format!("window {}", project.windows.len() + 1);
        let window = self.new_window_for_cwd(cwd, name)?;

        let project = &mut self.projects[active_project];
        project.windows.push(window);
        project.active_window = project.windows.len() - 1;
        self.resize_all_panes()
    }

    fn new_window_for_cwd(&mut self, cwd: PathBuf, name: String) -> Result<WindowPage> {
        Ok(WindowPage {
            name,
            panes: vec![self.spawn_pane(cwd, "pane 1".to_string())?],
            active_pane: 0,
            split_direction: SplitDirection::Vertical,
        })
    }

    fn split_pane(&mut self, direction: SplitDirection) -> Result<()> {
        let active_project = self.active_project;
        let Some(project) = self.projects.get(active_project) else {
            return self.new_project();
        };
        let Some(window) = project.windows.get(project.active_window) else {
            return self.new_window();
        };

        let cwd = self
            .active_terminal()
            .map(|pane| normalize_cwd(&pane.cwd))
            .unwrap_or_else(|| normalize_cwd(&project.cwd));
        let name = format!("pane {}", window.panes.len() + 1);
        let pane = self.spawn_pane(cwd, name)?;

        let project = &mut self.projects[active_project];
        let window = &mut project.windows[project.active_window];
        window.split_direction = direction;
        window.panes.push(pane);
        window.active_pane = window.panes.len() - 1;
        self.resize_all_panes()
    }

    fn spawn_pane(&mut self, cwd: PathBuf, name: String) -> Result<TerminalTab> {
        let id = self.allocate_tab_id();
        TerminalTab::new_default(
            id,
            cwd,
            name,
            self.terminal_cols,
            self.terminal_rows,
            self.output_tx.clone(),
        )
    }

    fn next_project(&mut self) {
        if !self.projects.is_empty() {
            self.active_project = (self.active_project + 1) % self.projects.len();
        }
    }

    fn previous_project(&mut self) {
        if !self.projects.is_empty() {
            self.active_project = if self.active_project == 0 {
                self.projects.len() - 1
            } else {
                self.active_project - 1
            };
        }
    }

    fn select_project(&mut self, index: usize) {
        if index < self.projects.len() {
            self.active_project = index;
        }
    }

    fn next_window(&mut self) {
        if let Some(project) = self.projects.get_mut(self.active_project)
            && !project.windows.is_empty()
        {
            project.active_window = (project.active_window + 1) % project.windows.len();
        }
    }

    fn previous_window(&mut self) {
        if let Some(project) = self.projects.get_mut(self.active_project)
            && !project.windows.is_empty()
        {
            project.active_window = if project.active_window == 0 {
                project.windows.len() - 1
            } else {
                project.active_window - 1
            };
        }
    }

    fn next_pane(&mut self) {
        if let Some(window) = self.active_window_mut()
            && !window.panes.is_empty()
        {
            window.active_pane = (window.active_pane + 1) % window.panes.len();
        }
    }

    fn previous_pane(&mut self) {
        if let Some(window) = self.active_window_mut()
            && !window.panes.is_empty()
        {
            window.active_pane = if window.active_pane == 0 {
                window.panes.len() - 1
            } else {
                window.active_pane - 1
            };
        }
    }

    fn close_active_pane(&mut self) -> Result<()> {
        let Some(window) = self.active_window_mut() else {
            return Ok(());
        };

        if window.panes.len() <= 1 {
            bail!("cannot close the last pane in a window");
        }

        window.panes.remove(window.active_pane);
        window.active_pane = window.active_pane.min(window.panes.len() - 1);
        self.resize_all_panes()
    }

    fn start_project_rename(&mut self) {
        let buffer = self
            .active_project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        self.rename_state = RenameState::Project { buffer };
    }

    fn start_window_rename(&mut self) {
        let buffer = self
            .active_window()
            .map(|window| window.name.clone())
            .unwrap_or_default();
        self.rename_state = RenameState::Window { buffer };
    }

    fn start_pane_rename(&mut self) {
        let buffer = self
            .active_terminal()
            .map(|pane| pane.name.clone())
            .unwrap_or_default();
        self.rename_state = RenameState::Pane { buffer };
    }

    fn active_window_mut(&mut self) -> Option<&mut WindowPage> {
        self.projects
            .get_mut(self.active_project)
            .and_then(|project| project.windows.get_mut(project.active_window))
    }

    fn write_to_active(&mut self, bytes: &[u8]) -> Result<()> {
        if let Some(window) = self.active_window_mut()
            && let Some(pane) = window.panes.get_mut(window.active_pane)
        {
            pane.write_input(bytes)?;
        }
        Ok(())
    }

    fn drain_pty_output(&mut self) {
        while let Ok((id, bytes)) = self.output_rx.try_recv() {
            for project in &mut self.projects {
                for window in &mut project.windows {
                    if let Some(pane) = window.panes.iter_mut().find(|pane| pane.id == id) {
                        pane.process_output(&bytes);
                        break;
                    }
                }
            }
        }
    }

    fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.terminal_cols = cols.saturating_sub(SIDEBAR_WIDTH).max(1);
        self.terminal_rows = rows.saturating_sub(3).max(1);
        self.resize_all_panes()
    }

    fn resize_all_panes(&mut self) -> Result<()> {
        for project in &mut self.projects {
            for window in &mut project.windows {
                let pane_count = window.panes.len().max(1) as u16;
                let (cols, rows) = pane_size(
                    self.terminal_cols,
                    self.terminal_rows,
                    window.split_direction,
                    pane_count,
                );
                for pane in &mut window.panes {
                    pane.resize(cols, rows)?;
                }
            }
        }
        Ok(())
    }

    fn allocate_tab_id(&mut self) -> u64 {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        id
    }
}

impl Project {
    fn session_project(&self) -> SessionProject {
        SessionProject {
            name: self.name.clone(),
            cwd: self.cwd.clone(),
            active_window: self.active_window,
            windows: self.windows.iter().map(WindowPage::session_window).collect(),
        }
    }
}

impl WindowPage {
    fn session_window(&self) -> SessionWindow {
        SessionWindow {
            name: self.name.clone(),
            active_pane: self.active_pane,
            split_direction: self.split_direction,
            panes: self.panes.iter().map(TerminalTab::session_tab).collect(),
        }
    }
}

fn apply_rename_key(key: KeyEvent, buffer: &mut String, mut apply: impl FnMut(String)) -> bool {
    match key.code {
        KeyCode::Enter => {
            let name = buffer.trim().to_string();
            if !name.is_empty() {
                apply(name);
            }
        }
        KeyCode::Esc => {}
        KeyCode::Backspace => {
            buffer.pop();
            return true;
        }
        KeyCode::Char(ch) => {
            buffer.push(ch);
            return true;
        }
        _ => return true,
    }
    true
}

fn pane_size(cols: u16, rows: u16, direction: SplitDirection, pane_count: u16) -> (u16, u16) {
    match direction {
        SplitDirection::Vertical => ((cols / pane_count).max(1), rows.max(1)),
        SplitDirection::Horizontal => (cols.max(1), (rows / pane_count).max(1)),
    }
}

fn project_name(cwd: &Path) -> String {
    cwd.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project")
        .to_string()
}

#[allow(dead_code)]
fn _encoded_key_for_tests(key: KeyEvent) -> Option<Vec<u8>> {
    encode_key(key)
}
