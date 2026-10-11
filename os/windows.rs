//! Windows keyboard input.
//!
//! This is the whole `keyboard` crate. It answers three questions:
//!
//! 1. *Is this key held down?* → [`Key::is_pressed`]
//! 2. *What keys are held down?* → [`pressed_keys`]
//! 3. *What am I typing?* → [`read_char`]
//!
//! All of it is built on one Win32 call,
//! [`GetAsyncKeyState`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate),
//! which reads the *global* keyboard state – no window, focus, or message
//! loop required.
//!
//! # Example
//!
//! ```no_run
//! use keyboard::{Key, pressed_keys, read_char, wait_for_key};
//!
//! // Is a key held down right now?
//! if Key::Space.is_pressed() {
//!     println!("jumping!");
//! }
//!
//! // What keys are held down right now?
//! for key in pressed_keys() {
//!     println!("{key:?} is down");
//! }
//!
//! // What character am I typing? ('\0' if nothing printable)
//! let c = read_char();
//! if c != '\0' {
//!     print!("{c}");
//! }
//!
//! // Wait for the user to press any key.
//! let key = wait_for_key();
//! println!("You pressed {key}");
//! ```

use std::fmt;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState,
    // Letters and digits.
    VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A, VK_B,
    VK_C, VK_D, VK_E, VK_F, VK_G, VK_H, VK_I, VK_J, VK_K, VK_L, VK_M, VK_N,
    VK_O, VK_P, VK_Q, VK_R, VK_S, VK_T, VK_U, VK_V, VK_W, VK_X, VK_Y, VK_Z,
    // Punctuation (US layout).
    VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7,
    VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS,
    // Everything else.
    VK_APPS, VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END,
    VK_ESCAPE, VK_F1, VK_F10, VK_F11, VK_F12, VK_F2, VK_F3, VK_F4, VK_F5,
    VK_F6, VK_F7, VK_F8, VK_F9, VK_HOME, VK_INSERT, VK_LEFT, VK_LWIN,
    VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE,
    VK_TAB, VK_UP,
};

/// A single key on the keyboard.
///
/// This is the only type you need to know. Compare keys with `==`, print them
/// with `{}`, and ask them to do things:
///
/// ```
/// use keyboard::Key;
///
/// assert_eq!(Key::A.to_char(), 'a');
/// assert_eq!(Key::Digit1.to_char(), '1');
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)] // `Key::A` means "the A key" – no more to say.
pub enum Key {
    // Letters.
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers.
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,

    // Punctuation.
    Minus, Equal, BracketLeft, BracketRight, Backslash, Semicolon, Quote,
    Backquote, Comma, Period, Slash,

    // Keys you press to make things happen.
    Space, Tab, Enter, Backspace, Escape, Insert, Delete,

    // Cursor movement.
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Home, End, PageUp, PageDown,

    // The row of F-keys at the top.
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Modifier keys.
    Shift, Control, Alt, Win, Menu,

    // Toggles.
    CapsLock,
}

impl Key {
    /// Returns `true` if this key is held down right now.
    ///
    /// This checks the whole keyboard, so it works even when your program
    /// doesn't have a window or focus.
    ///
    /// ```no_run
    /// use keyboard::Key;
    ///
    /// if Key::Space.is_pressed() {
    ///     println!("jumping!");
    /// }
    /// ```
    #[inline]
    pub fn is_pressed(self) -> bool {
        is_pressed_vk(self.to_vk())
    }

    /// Returns the character this key types, or `'\0'` if it types nothing.
    ///
    /// Shift and Caps Lock are applied automatically, so `Key::A.to_char()`
    /// is `'a'` normally, `'A'` with Shift or Caps Lock, and back to `'a'`
    /// if both are on. The layout is standard US (ANSI).
    ///
    /// ```
    /// use keyboard::Key;
    ///
    /// assert_eq!(Key::G.to_char(), 'g'); // ...normally!
    /// assert_eq!(Key::Space.to_char(), ' ');
    /// assert_eq!(Key::ArrowUp.to_char(), '\0');
    /// ```
    pub fn to_char(self) -> char {
        // What does this key make on its own? Letter keys come back as
        // lowercase ASCII — that's how we spot them.
        let plain = char_for(self, false);
        if plain.is_ascii_lowercase() {
            // Letters are flipped by Shift and Caps Lock.
            char_for(self, Key::Shift.is_pressed() ^ is_caps_locked())
        } else {
            // Digits and punctuation only care about Shift.
            char_for(self, Key::Shift.is_pressed())
        }
    }

    /// Returns the Windows *virtual-key code* for this key.
    ///
    /// You usually don't need this; it's an escape hatch for talking to other
    /// Windows APIs that want a `VK_*` code.
    ///
    /// ```
    /// use keyboard::Key;
    ///
    /// assert_eq!(Key::A.to_vk(), 0x41);
    /// assert_eq!(Key::Enter.to_vk(), 0x0D);
    /// ```
    #[inline]
    pub fn to_vk(self) -> u16 {
        match self {
            Key::A => VK_A.0,
            Key::B => VK_B.0,
            Key::C => VK_C.0,
            Key::D => VK_D.0,
            Key::E => VK_E.0,
            Key::F => VK_F.0,
            Key::G => VK_G.0,
            Key::H => VK_H.0,
            Key::I => VK_I.0,
            Key::J => VK_J.0,
            Key::K => VK_K.0,
            Key::L => VK_L.0,
            Key::M => VK_M.0,
            Key::N => VK_N.0,
            Key::O => VK_O.0,
            Key::P => VK_P.0,
            Key::Q => VK_Q.0,
            Key::R => VK_R.0,
            Key::S => VK_S.0,
            Key::T => VK_T.0,
            Key::U => VK_U.0,
            Key::V => VK_V.0,
            Key::W => VK_W.0,
            Key::X => VK_X.0,
            Key::Y => VK_Y.0,
            Key::Z => VK_Z.0,

            Key::Digit0 => VK_0.0,
            Key::Digit1 => VK_1.0,
            Key::Digit2 => VK_2.0,
            Key::Digit3 => VK_3.0,
            Key::Digit4 => VK_4.0,
            Key::Digit5 => VK_5.0,
            Key::Digit6 => VK_6.0,
            Key::Digit7 => VK_7.0,
            Key::Digit8 => VK_8.0,
            Key::Digit9 => VK_9.0,

            Key::Minus => VK_OEM_MINUS.0,
            Key::Equal => VK_OEM_PLUS.0,
            Key::BracketLeft => VK_OEM_4.0,
            Key::BracketRight => VK_OEM_6.0,
            Key::Backslash => VK_OEM_5.0,
            Key::Semicolon => VK_OEM_1.0,
            Key::Quote => VK_OEM_7.0,
            Key::Backquote => VK_OEM_3.0,
            Key::Comma => VK_OEM_COMMA.0,
            Key::Period => VK_OEM_PERIOD.0,
            Key::Slash => VK_OEM_2.0,

            Key::Space => VK_SPACE.0,
            Key::Tab => VK_TAB.0,
            Key::Enter => VK_RETURN.0,
            Key::Backspace => VK_BACK.0,
            Key::Escape => VK_ESCAPE.0,
            Key::Insert => VK_INSERT.0,
            Key::Delete => VK_DELETE.0,

            Key::ArrowUp => VK_UP.0,
            Key::ArrowDown => VK_DOWN.0,
            Key::ArrowLeft => VK_LEFT.0,
            Key::ArrowRight => VK_RIGHT.0,
            Key::Home => VK_HOME.0,
            Key::End => VK_END.0,
            Key::PageUp => VK_PRIOR.0,
            Key::PageDown => VK_NEXT.0,

            Key::F1 => VK_F1.0,
            Key::F2 => VK_F2.0,
            Key::F3 => VK_F3.0,
            Key::F4 => VK_F4.0,
            Key::F5 => VK_F5.0,
            Key::F6 => VK_F6.0,
            Key::F7 => VK_F7.0,
            Key::F8 => VK_F8.0,
            Key::F9 => VK_F9.0,
            Key::F10 => VK_F10.0,
            Key::F11 => VK_F11.0,
            Key::F12 => VK_F12.0,

            Key::Shift => VK_SHIFT.0,
            Key::Control => VK_CONTROL.0,
            Key::Alt => VK_MENU.0,
            Key::Win => VK_LWIN.0,
            Key::Menu => VK_APPS.0,

            Key::CapsLock => VK_CAPITAL.0,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Every key on the keyboard, in the order `pressed_keys()` reports them.
///
/// Letter and number keys come first, so "what am I typing?"-style reads
/// naturally favour them over modifiers.
const ALL_KEYS: &[Key] = &[
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I,
    Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R,
    Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
    Key::Digit0, Key::Digit1, Key::Digit2, Key::Digit3, Key::Digit4,
    Key::Digit5, Key::Digit6, Key::Digit7, Key::Digit8, Key::Digit9,
    Key::Minus, Key::Equal, Key::BracketLeft, Key::BracketRight,
    Key::Backslash, Key::Semicolon, Key::Quote, Key::Backquote, Key::Comma,
    Key::Period, Key::Slash,
    Key::Space, Key::Tab, Key::Enter, Key::Backspace, Key::Escape,
    Key::Insert, Key::Delete,
    Key::ArrowUp, Key::ArrowDown, Key::ArrowLeft, Key::ArrowRight, Key::Home,
    Key::End, Key::PageUp, Key::PageDown,
    Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6, Key::F7, Key::F8,
    Key::F9, Key::F10, Key::F11, Key::F12,
    Key::Shift, Key::Control, Key::Alt, Key::Win, Key::Menu, Key::CapsLock,
];

/// Returns every key that is held down right now.
///
/// Typing keys (letters, numbers, punctuation) come first, modifiers last.
///
/// ```no_run
/// use keyboard::pressed_keys;
///
/// if pressed_keys().contains(&keyboard::Key::Control) {
///     println!("ctrl is down");
/// }
/// ```
pub fn pressed_keys() -> Vec<Key> {
    ALL_KEYS.iter().copied().filter(|key| key.is_pressed()).collect()
}

/// Returns the character being typed right now, or `'\0'` if nothing
/// printable is pressed.
///
/// Shift and Caps Lock are applied automatically, so holding Shift and
/// pressing `A` returns `'A'`. It's the simplest possible way to read text:
///
/// ```no_run
/// use keyboard::read_char;
///
/// let c = read_char();
/// if c != '\0' {
///     print!("{c}");
/// }
/// ```
pub fn read_char() -> char {
    ALL_KEYS
        .iter()
        .copied()
        .map(Key::to_char)
        .find(|c| *c != '\0')
        .unwrap_or('\0')
}

/// Waits until a key is pressed, then returns it.
///
/// Blocks the current thread, checking every 10 ms. Perfect for "press any
/// key to continue" prompts and simple menus:
///
/// ```no_run
/// use keyboard::wait_for_key;
///
/// println!("Press any key to continue...");
/// let key = wait_for_key();
/// println!("You pressed {key}");
/// ```
pub fn wait_for_key() -> Key {
    loop {
        if let Some(key) = pressed_keys().into_iter().next() {
            return key;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Whether a virtual-key *down-state* bit is set right now.
///
/// # Safety
///
/// `GetAsyncKeyState` only reads global keyboard state: it cannot fail and
/// never blocks. `vk` always comes from [`Key::to_vk`] or a `VK_*` constant,
/// so the call is sound.
#[inline]
fn is_pressed_vk(vk: u16) -> bool {
    // The high bit of the returned i16 is set while the key is down.
    unsafe { GetAsyncKeyState(i32::from(vk)) < 0 }
}

/// Whether Caps Lock is toggled on.
///
/// # Safety
///
/// `GetKeyState` only reads global key state and cannot fail.
#[inline]
fn is_caps_locked() -> bool {
    // The low bit of the returned i16 is set when the toggle is on.
    unsafe { GetKeyState(i32::from(VK_CAPITAL.0)) & 1 == 1 }
}

/// The character a key makes, from a fixed US layout.
///
/// `shift` is passed in explicitly so this can be unit-tested without
/// touching real hardware.
fn char_for(key: Key, shift: bool) -> char {
    match key {
        Key::A => if shift { 'A' } else { 'a' },
        Key::B => if shift { 'B' } else { 'b' },
        Key::C => if shift { 'C' } else { 'c' },
        Key::D => if shift { 'D' } else { 'd' },
        Key::E => if shift { 'E' } else { 'e' },
        Key::F => if shift { 'F' } else { 'f' },
        Key::G => if shift { 'G' } else { 'g' },
        Key::H => if shift { 'H' } else { 'h' },
        Key::I => if shift { 'I' } else { 'i' },
        Key::J => if shift { 'J' } else { 'j' },
        Key::K => if shift { 'K' } else { 'k' },
        Key::L => if shift { 'L' } else { 'l' },
        Key::M => if shift { 'M' } else { 'm' },
        Key::N => if shift { 'N' } else { 'n' },
        Key::O => if shift { 'O' } else { 'o' },
        Key::P => if shift { 'P' } else { 'p' },
        Key::Q => if shift { 'Q' } else { 'q' },
        Key::R => if shift { 'R' } else { 'r' },
        Key::S => if shift { 'S' } else { 's' },
        Key::T => if shift { 'T' } else { 't' },
        Key::U => if shift { 'U' } else { 'u' },
        Key::V => if shift { 'V' } else { 'v' },
        Key::W => if shift { 'W' } else { 'w' },
        Key::X => if shift { 'X' } else { 'x' },
        Key::Y => if shift { 'Y' } else { 'y' },
        Key::Z => if shift { 'Z' } else { 'z' },

        Key::Digit0 => if shift { ')' } else { '0' },
        Key::Digit1 => if shift { '!' } else { '1' },
        Key::Digit2 => if shift { '@' } else { '2' },
        Key::Digit3 => if shift { '#' } else { '3' },
        Key::Digit4 => if shift { '$' } else { '4' },
        Key::Digit5 => if shift { '%' } else { '5' },
        Key::Digit6 => if shift { '^' } else { '6' },
        Key::Digit7 => if shift { '&' } else { '7' },
        Key::Digit8 => if shift { '*' } else { '8' },
        Key::Digit9 => if shift { '(' } else { '9' },

        Key::Minus => if shift { '_' } else { '-' },
        Key::Equal => if shift { '+' } else { '=' },
        Key::BracketLeft => if shift { '{' } else { '[' },
        Key::BracketRight => if shift { '}' } else { ']' },
        Key::Backslash => if shift { '|' } else { '\\' },
        Key::Semicolon => if shift { ':' } else { ';' },
        Key::Quote => if shift { '"' } else { '\'' },
        Key::Backquote => if shift { '~' } else { '`' },
        Key::Comma => if shift { '<' } else { ',' },
        Key::Period => if shift { '>' } else { '.' },
        Key::Slash => if shift { '?' } else { '/' },

        Key::Space => ' ',
        _ => '\0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_key_codes_are_right() {
        assert_eq!(Key::A.to_vk(), VK_A.0);
        assert_eq!(Key::Z.to_vk(), VK_Z.0);
        assert_eq!(Key::Digit0.to_vk(), VK_0.0);
        assert_eq!(Key::Digit9.to_vk(), VK_9.0);
        assert_eq!(Key::Enter.to_vk(), VK_RETURN.0);
        assert_eq!(Key::Space.to_vk(), VK_SPACE.0);
        assert_eq!(Key::ArrowUp.to_vk(), VK_UP.0);
        assert_eq!(Key::F12.to_vk(), VK_F12.0);
        assert_eq!(Key::Shift.to_vk(), VK_SHIFT.0);
        assert_eq!(Key::Win.to_vk(), VK_LWIN.0);
        assert_eq!(Key::CapsLock.to_vk(), VK_CAPITAL.0);
    }

    #[test]
    fn letters_follow_shift() {
        for key in [
            Key::A, Key::B, Key::C, Key::G, Key::J, Key::K, Key::M, Key::R,
            Key::X, Key::Y, Key::Z,
        ] {
            let lower = format!("{key:?}").to_ascii_lowercase();
            let upper = lower.to_ascii_uppercase();
            assert_eq!(
                char_for(key, false).to_string(),
                lower,
                "plain {key:?}"
            );
            assert_eq!(
                char_for(key, true).to_string(),
                upper,
                "shifted {key:?}"
            );
        }
    }

    #[test]
    fn punctuation_is_us_layout() {
        for (key, plain, shifted) in [
            (Key::Digit1, '1', '!'),
            (Key::Digit8, '8', '*'),
            (Key::Minus, '-', '_'),
            (Key::Equal, '=', '+'),
            (Key::BracketLeft, '[', '{'),
            (Key::BracketRight, ']', '}'),
            (Key::Backslash, '\\', '|'),
            (Key::Semicolon, ';', ':'),
            (Key::Quote, '\'', '"'),
            (Key::Backquote, '`', '~'),
            (Key::Comma, ',', '<'),
            (Key::Period, '.', '>'),
            (Key::Slash, '/', '?'),
            (Key::Space, ' ', ' '),
        ] {
            assert_eq!(char_for(key, false), plain, "{key:?}");
            assert_eq!(char_for(key, true), shifted, "{key:?}");
        }
    }

    #[test]
    fn other_keys_type_nothing() {
        for key in [
            Key::ArrowUp, Key::Tab, Key::Enter, Key::Backspace, Key::Escape,
            Key::Insert, Key::Delete, Key::F5, Key::Shift, Key::Control,
            Key::Alt, Key::Win, Key::Menu, Key::CapsLock,
        ] {
            assert_eq!(char_for(key, false), '\0', "{key:?}");
            assert_eq!(char_for(key, true), '\0', "{key:?}");
        }
    }

    #[test]
    fn display_matches_debug() {
        assert_eq!(Key::ArrowUp.to_string(), "ArrowUp");
        assert_eq!(Key::Space.to_string(), "Space");
    }
}