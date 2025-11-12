#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KeyCode {
    Char(u8), // printable ASCII
    Enter,
    Backspace,
    Tab,
    Unknown(u8),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Modifiers(u8);

impl Modifiers {
    pub const SHIFT: u8 = 1 << 0;
    pub const CTRL: u8 = 1 << 1;
    pub const ALT: u8 = 1 << 2;
    pub const CAPS: u8 = 1 << 3;

    pub const fn empty() -> Self {
        Self(0)
    }
    pub fn insert(&mut self, mask: u8) {
        self.0 |= mask;
    }
    pub fn remove(&mut self, mask: u8) {
        self.0 &= !mask;
    }
    pub const fn contains(self, mask: u8) -> bool {
        (self.0 & mask) != 0
    }
    pub const fn bits(self) -> u8 {
        self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub mods: Modifiers,
    pub pressed: bool, // true = make, false = break
}

impl KeyEvent {
    /// Returns a byte to echo (ASCII / Enter / Tab / Backspace) when relevant.
    pub fn printable_byte(self) -> Option<u8> {
        match self.code {
            KeyCode::Char(b) if self.pressed => Some(b),
            KeyCode::Enter if self.pressed => Some(b'\n'),
            KeyCode::Backspace if self.pressed => Some(0x08),
            KeyCode::Tab if self.pressed => Some(b'\t'),
            _ => None,
        }
    }
}
