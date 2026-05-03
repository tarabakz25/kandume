use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SplitDirection {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum SessionPaneLayout {
    Leaf(usize),
    Split {
        direction: SplitDirection,
        first: Box<SessionPaneLayout>,
        second: Box<SessionPaneLayout>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionTab {
    pub(crate) name: String,
    pub(crate) cwd: PathBuf,
    pub(crate) shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionWindow {
    pub(crate) name: String,
    pub(crate) active_pane: usize,
    pub(crate) split_direction: SplitDirection,
    #[serde(default)]
    pub(crate) layout: Option<SessionPaneLayout>,
    pub(crate) panes: Vec<SessionTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionProject {
    pub(crate) name: String,
    pub(crate) cwd: PathBuf,
    pub(crate) active_window: usize,
    pub(crate) windows: Vec<SessionWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionState {
    pub(crate) active_project: usize,
    pub(crate) projects: Vec<SessionProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectSessionsState {
    active_project: usize,
    projects: Vec<ProjectSessionsProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectSessionsProject {
    name: String,
    cwd: PathBuf,
    active_session: usize,
    sessions: Vec<SessionTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacySessionState {
    active_tab: usize,
    tabs: Vec<SessionTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum StoredSessionState {
    Current(SessionState),
    ProjectSessions(ProjectSessionsState),
    Legacy(LegacySessionState),
}

pub(crate) fn load() -> Result<Option<SessionState>> {
    let path = session_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read session file {}", path.display()))?;
    let session: StoredSessionState = toml::from_str(&content)
        .with_context(|| format!("failed to parse session file {}", path.display()))?;
    Ok(Some(match session {
        StoredSessionState::Current(session) => session,
        StoredSessionState::ProjectSessions(session) => migrate_project_sessions(session),
        StoredSessionState::Legacy(session) => migrate_legacy_session(session),
    }))
}

pub(crate) fn save(session: &SessionState) -> Result<()> {
    let path = session_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(session).context("failed to serialize session")?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write session file {}", path.display()))
}

fn session_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("failed to resolve user config directory")?;
    Ok(config_dir.join("kandume").join("session.toml"))
}

fn migrate_project_sessions(session: ProjectSessionsState) -> SessionState {
    SessionState {
        active_project: session.active_project,
        projects: session
            .projects
            .into_iter()
            .map(|project| SessionProject {
                name: project.name,
                cwd: project.cwd,
                active_window: 0,
                windows: vec![SessionWindow {
                    name: "window 1".to_string(),
                    active_pane: project.active_session,
                    split_direction: SplitDirection::Vertical,
                    layout: None,
                    panes: project.sessions,
                }],
            })
            .collect(),
    }
}

fn migrate_legacy_session(session: LegacySessionState) -> SessionState {
    let cwd = session
        .tabs
        .first()
        .map(|tab| tab.cwd.clone())
        .unwrap_or_default();
    let name = cwd
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project")
        .to_string();

    SessionState {
        active_project: 0,
        projects: vec![SessionProject {
            name,
            cwd,
            active_window: 0,
            windows: vec![SessionWindow {
                name: "window 1".to_string(),
                active_pane: session.active_tab,
                split_direction: SplitDirection::Vertical,
                layout: None,
                panes: session.tabs,
            }],
        }],
    }
}
