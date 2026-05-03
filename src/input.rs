use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InputAction {
    Send(Vec<u8>),
    NewProject,
    NewWindow,
    SplitVertical,
    SplitHorizontal,
    NextProject,
    PreviousProject,
    SelectProject(usize),
    NextWindow,
    PreviousWindow,
    NextPane,
    PreviousPane,
    ClosePane,
    StartProjectRename,
    StartWindowRename,
    StartPaneRename,
    SaveAndQuit,
    Quit,
    ToggleHelp,
    None,
}

#[derive(Debug, Default)]
pub(crate) struct InputState {
    prefix: bool,
}

impl InputState {
    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> InputAction {
        if self.prefix {
            self.prefix = false;
            return match key.code {
                KeyCode::Char('t') => InputAction::NewProject,
                KeyCode::Char('c') => InputAction::NewWindow,
                KeyCode::Char('%') => InputAction::SplitVertical,
                KeyCode::Char('"') => InputAction::SplitHorizontal,
                KeyCode::Char('n') => InputAction::NextProject,
                KeyCode::Char('p') => InputAction::PreviousProject,
                KeyCode::Char(']') => InputAction::NextWindow,
                KeyCode::Char('[') => InputAction::PreviousWindow,
                KeyCode::Char('o') => InputAction::NextPane,
                KeyCode::Char(';') => InputAction::PreviousPane,
                KeyCode::Char('x') => InputAction::ClosePane,
                KeyCode::Char(',') => InputAction::StartProjectRename,
                KeyCode::Char('.') => InputAction::StartWindowRename,
                KeyCode::Char('r') => InputAction::StartPaneRename,
                KeyCode::Char('d') => InputAction::SaveAndQuit,
                KeyCode::Char('q') => InputAction::Quit,
                KeyCode::Char('?') => InputAction::ToggleHelp,
                KeyCode::Char(ch) if ('1'..='9').contains(&ch) => {
                    InputAction::SelectProject(ch as usize - '1' as usize)
                }
                KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    InputAction::Send(vec![0x02])
                }
                _ => InputAction::None,
            };
        }

        if key.code == KeyCode::Char('b') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.prefix = true;
            return InputAction::None;
        }

        encode_key(key).map_or(InputAction::None, InputAction::Send)
    }

    pub(crate) fn is_prefix_active(&self) -> bool {
        self.prefix
    }
}

pub(crate) fn encode_key(key: KeyEvent) -> Option<Vec<u8>> {
    match key.code {
        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            ctrl_byte(ch).map(|byte| vec![byte])
        }
        KeyCode::Char(ch) => Some(ch.to_string().into_bytes()),
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::BackTab => Some(b"\x1b[Z".to_vec()),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
        KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::F(n) => encode_function_key(n),
        KeyCode::Null
        | KeyCode::CapsLock
        | KeyCode::ScrollLock
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::Menu
        | KeyCode::KeypadBegin
        | KeyCode::Media(_)
        | KeyCode::Modifier(_) => None,
    }
}

fn ctrl_byte(ch: char) -> Option<u8> {
    let lower = ch.to_ascii_lowercase();
    if lower.is_ascii_lowercase() {
        Some(lower as u8 - b'a' + 1)
    } else {
        match ch {
            '[' => Some(0x1b),
            '\\' => Some(0x1c),
            ']' => Some(0x1d),
            '^' => Some(0x1e),
            '_' => Some(0x1f),
            _ => None,
        }
    }
}

fn encode_function_key(n: u8) -> Option<Vec<u8>> {
    let sequence = match n {
        1 => "\x1bOP",
        2 => "\x1bOQ",
        3 => "\x1bOR",
        4 => "\x1bOS",
        5 => "\x1b[15~",
        6 => "\x1b[17~",
        7 => "\x1b[18~",
        8 => "\x1b[19~",
        9 => "\x1b[20~",
        10 => "\x1b[21~",
        11 => "\x1b[23~",
        12 => "\x1b[24~",
        _ => return None,
    };
    Some(sequence.as_bytes().to_vec())
}
