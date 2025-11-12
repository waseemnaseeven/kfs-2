use core::fmt;

pub trait Console {
    fn clear_screen(&mut self);
    fn set_color(&mut self, fg: u8, bg: u8);
    fn write_byte(&mut self, b: u8);
    fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // ▮
            }
        }
    }
}

pub use crate::drivers::video::vga_text as vga;

use vga::try_with_console;

pub fn _print(args: fmt::Arguments) {
    try_with_console(|c| {
        use core::fmt::Write;
        struct Adaptor<'a, C: Console>(&'a mut C);
        impl<'a, C: Console> fmt::Write for Adaptor<'a, C> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0.write_str(s);
                Ok(())
            }
        }
        let _ = Adaptor(c).write_fmt(args);
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::subsystems::console::_print(format_args!($($arg)*)) }
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) }
}

pub fn init() {
    try_with_console(|c| {
        c.set_color(0x07, 0x00);
        c.clear_screen();
    });
}

pub fn write_byte(b: u8) {
    try_with_console(|c| c.write_byte(b));
}

pub fn write_str_fast(s: &str) {
    try_with_console(|c| c.write_bytes(s.as_bytes()));
}
pub fn backspace() {
    try_with_console(|c| c.backspace());
}

/// Temporarily set color for the duration of `f`, then restore previous color.
pub fn with_color<F: FnOnce()>(fg: u8, bg: u8, f: F) {
    try_with_console(|c| {
        let old = c.get_color_code();
        c.set_color(fg, bg);
        f();
        let old_fg = old & 0x0F;
        let old_bg = (old >> 4) & 0x0F;
        c.set_color(old_fg, old_bg);
    });
}
