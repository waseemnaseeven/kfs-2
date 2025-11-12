use super::types::{KeyCode, KeyEvent, Modifiers};
use crate::drivers::bus::ps2::controller as ctl;

fn is_break(sc: u8) -> bool {
    sc & 0x80 != 0
}

pub struct State {
    pub mods: Modifiers,
}
impl State {
    pub const fn new() -> Self {
        Self {
            mods: Modifiers::empty(),
        }
    }
}

impl State {
    fn on_make(&mut self, sc: u8) -> KeyEvent {
        match sc {
            0x2A | 0x36 => {
                self.mods.insert(Modifiers::SHIFT);
                KeyEvent {
                    code: KeyCode::Unknown(sc),
                    mods: self.mods,
                    pressed: true,
                }
            }
            0x1C => KeyEvent {
                code: KeyCode::Enter,
                mods: self.mods,
                pressed: true,
            },
            0x0E => KeyEvent {
                code: KeyCode::Backspace,
                mods: self.mods,
                pressed: true,
            },
            0x0F => KeyEvent {
                code: KeyCode::Tab,
                mods: self.mods,
                pressed: true,
            },
            _ => {
                if let Some(code) = super::keymap_us::translate_printable(sc, self.mods) {
                    KeyEvent {
                        code,
                        mods: self.mods,
                        pressed: true,
                    }
                } else {
                    KeyEvent {
                        code: KeyCode::Unknown(sc),
                        mods: self.mods,
                        pressed: true,
                    }
                }
            }
        }
    }
    fn on_break(&mut self, sc: u8) -> KeyEvent {
        match sc & 0x7F {
            0x2A | 0x36 => {
                self.mods.remove(Modifiers::SHIFT);
                KeyEvent {
                    code: KeyCode::Unknown(sc),
                    mods: self.mods,
                    pressed: false,
                }
            }
            0x1C => KeyEvent {
                code: KeyCode::Enter,
                mods: self.mods,
                pressed: false,
            },
            0x0E => KeyEvent {
                code: KeyCode::Backspace,
                mods: self.mods,
                pressed: false,
            },
            0x0F => KeyEvent {
                code: KeyCode::Tab,
                mods: self.mods,
                pressed: false,
            },
            _ => KeyEvent {
                code: KeyCode::Unknown(sc),
                mods: self.mods,
                pressed: false,
            },
        }
    }
}

pub fn poll_once(st: &mut State) -> Option<KeyEvent> {
    if !ctl::data_available() {
        return None;
    }
    let sc = ctl::read_data();
    Some(if is_break(sc) {
        st.on_break(sc)
    } else {
        st.on_make(sc)
    })
}
