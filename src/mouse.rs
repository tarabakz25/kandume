use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};

/// Encode an event into XTerm SGR (1006) mouse reporting as written to the PTY.
///
/// `local_col_0based` / `local_row_0based` are relative to the pane terminal viewport (top-left = 0).
/// Output coordinates are 1-based per XTerm.
pub(crate) fn encode_sgr_mouse(
    kind: MouseEventKind,
    modifiers: KeyModifiers,
    local_col_0based: u16,
    local_row_0based: u16,
    pane_cols: u16,
    pane_rows: u16,
) -> Option<Vec<u8>> {
    let cols = pane_cols.max(1);
    let rows = pane_rows.max(1);
    let max_c = cols.saturating_sub(1);
    let max_r = rows.saturating_sub(1);
    let lc = local_col_0based.min(max_c);
    let lr = local_row_0based.min(max_r);
    let cx = u32::from(lc) + 1;
    let cy = u32::from(lr) + 1;

    let (cb, lowercase_m) = mouse_cb(kind);
    let cb = apply_modifiers(cb, modifiers);

    let mut seq = format!("\x1b[<{cb};{cx};{cy}");
    seq.push(if lowercase_m { 'm' } else { 'M' });
    Some(seq.into_bytes())
}

fn apply_modifiers(mut cb: u8, modifiers: KeyModifiers) -> u8 {
    if modifiers.contains(KeyModifiers::SHIFT) {
        cb |= 0b0000_0100;
    }
    if modifiers.contains(KeyModifiers::ALT) {
        cb |= 0b0000_1000;
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        cb |= 0b0001_0000;
    }
    cb
}

/// Returns `(encoded_cb, use_lowercase_m_suffix)`.
fn mouse_cb(kind: MouseEventKind) -> (u8, bool) {
    match kind {
        MouseEventKind::Down(MouseButton::Left) => (0_u8, false),
        MouseEventKind::Down(MouseButton::Middle) => (1_u8, false),
        MouseEventKind::Down(MouseButton::Right) => (2_u8, false),
        MouseEventKind::Up(_) => (3_u8, true),
        MouseEventKind::Drag(MouseButton::Left) => (0x20_u8, false),
        MouseEventKind::Drag(MouseButton::Middle) => (1_u8 | 0x20, false),
        MouseEventKind::Drag(MouseButton::Right) => (2_u8 | 0x20, false),
        MouseEventKind::Moved => (3_u8 | 0x20, false),
        MouseEventKind::ScrollUp => (0x40_u8, false),
        MouseEventKind::ScrollDown => (0x41_u8, false),
        MouseEventKind::ScrollLeft => (0x42_u8, false),
        MouseEventKind::ScrollRight => (0x43_u8, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::MouseButton;

    #[test]
    fn encodes_left_press_sgr() {
        let bytes = encode_sgr_mouse(
            MouseEventKind::Down(MouseButton::Left),
            KeyModifiers::empty(),
            0,
            0,
            80,
            24,
        )
        .unwrap();
        assert_eq!(bytes, b"\x1b[<0;1;1M");
    }

    #[test]
    fn encodes_scroll_down_with_coords() {
        let bytes = encode_sgr_mouse(
            MouseEventKind::ScrollDown,
            KeyModifiers::empty(),
            10,
            5,
            80,
            24,
        )
        .unwrap();
        assert_eq!(bytes, b"\x1b[<65;11;6M");
    }

    #[test]
    fn encodes_release_lowercase_m() {
        let bytes = encode_sgr_mouse(
            MouseEventKind::Up(MouseButton::Left),
            KeyModifiers::empty(),
            3,
            2,
            80,
            24,
        )
        .unwrap();
        assert_eq!(bytes, b"\x1b[<3;4;3m");
    }

    #[test]
    fn clamps_to_pane_bounds() {
        let bytes = encode_sgr_mouse(
            MouseEventKind::Down(MouseButton::Left),
            KeyModifiers::empty(),
            99,
            99,
            10,
            5,
        )
        .unwrap();
        assert_eq!(bytes, b"\x1b[<0;10;5M");
    }

    #[test]
    fn shift_modifier_sets_bit() {
        let bytes = encode_sgr_mouse(
            MouseEventKind::Down(MouseButton::Left),
            KeyModifiers::SHIFT,
            0,
            0,
            80,
            24,
        )
        .unwrap();
        assert_eq!(bytes, b"\x1b[<4;1;1M");
    }
}
