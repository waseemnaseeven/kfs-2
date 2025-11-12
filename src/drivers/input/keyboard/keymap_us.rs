use super::scancode_set1::{MAP, MAP_SHIFT};
use super::types::{KeyCode, Modifiers};

pub fn translate_printable(sc: u8, mods: Modifiers) -> Option<KeyCode> {
    if (sc as usize) < MAP.len() {
        let shifted = mods.contains(Modifiers::SHIFT);
        let b = if shifted {
            MAP_SHIFT[sc as usize]
        } else {
            MAP[sc as usize]
        };
        if b >= 0x20 {
            return Some(KeyCode::Char(b));
        }
    }
    None
}
