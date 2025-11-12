mod keymap_us;
pub mod ps2;
mod scancode_set1;
pub mod types;

use types::KeyEvent;

static mut STATE: ps2::State = ps2::State::new();

pub fn poll_event() -> Option<KeyEvent> {
    unsafe { ps2::poll_once(&mut STATE) }
}
