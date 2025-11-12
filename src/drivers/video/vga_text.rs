use core::ptr::{read_volatile, write_volatile, NonNull};

use crate::arch::x86::port::outb;
use crate::subsystems::console::Console;
use crate::sync::spinlock::SpinLock;

pub const HEIGHT: usize = 25;
pub const WIDTH: usize = 80;

const VGA_BASE: usize = 0xb8000;
const VGA_CRTC_ADDR: u16 = 0x3D4;
const VGA_CRTC_DATA: u16 = 0x3D5;

pub mod vga_color {
    pub const BLACK: u8 = 0x0;
    pub const BLUE: u8 = 0x1;
    pub const GREEN: u8 = 0x2;
    pub const CYAN: u8 = 0x3;
    pub const RED: u8 = 0x4;
    pub const MAGENTA: u8 = 0x5;
    pub const BROWN: u8 = 0x6;
    pub const LIGHT_GRAY: u8 = 0x7;
    pub const DARK_GRAY: u8 = 0x8;
    pub const LIGHT_BLUE: u8 = 0x9;
    pub const LIGHT_GREEN: u8 = 0xA;
    pub const LIGHT_CYAN: u8 = 0xB;
    pub const LIGHT_RED: u8 = 0xC;
    pub const LIGHT_MAGENTA: u8 = 0xD;
    pub const YELLOW: u8 = 0xE;
    pub const WHITE: u8 = 0xF;
}

const fn color_code(fg: u8, bg: u8) -> u8 {
    ((bg & 0x0F) << 4) | (fg & 0x0F)
}

pub struct VgaTextConsole {
    row: usize,
    col: usize,
    color: u8,         // (bg<<4 | fg)
    buf: NonNull<u16>, // MMIO 0xb8000
}

impl VgaTextConsole {
    pub const fn new() -> Self {
        Self {
            row: 0,
            col: 0,
            color: color_code(7, 0), // LightGray on Black
            buf: NonNull::new(VGA_BASE as *mut u16).unwrap(),
        }
    }

    pub fn get_color_code(&self) -> u8 {
        self.color
    }

    fn pack(&self, ch: u8) -> u16 {
        ((self.color as u16) << 8) | ch as u16
    }

    unsafe fn cell_ptr(&self, row: usize, col: usize) -> *mut u16 {
        self.buf.as_ptr().add(row * WIDTH + col)
    }

    unsafe fn write_cell(&self, row: usize, col: usize, v: u16) {
        write_volatile(self.cell_ptr(row, col), v);
    }

    unsafe fn read_cell(&self, row: usize, col: usize) -> u16 {
        read_volatile(self.cell_ptr(row, col))
    }

    fn hw_cursor_update(&self) {
        let pos = (self.row * WIDTH + self.col) as u16;
        unsafe {
            outb(VGA_CRTC_ADDR, 0x0F);
            outb(VGA_CRTC_DATA, (pos & 0xFF) as u8);
            outb(VGA_CRTC_ADDR, 0x0E);
            outb(VGA_CRTC_DATA, (pos >> 8) as u8);
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row < HEIGHT - 1 {
            self.row += 1;
        } else {
            for r in 1..HEIGHT {
                for c in 0..WIDTH {
                    let v = unsafe { self.read_cell(r, c) };
                    unsafe { self.write_cell(r - 1, c, v) };
                }
            }
            self.clear_row(HEIGHT - 1);
            self.hw_cursor_update();
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ((self.color as u16) << 8) | (b' ' as u16);
        for col in 0..WIDTH {
            unsafe { self.write_cell(row, col, blank) };
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_byte(b);
        }
    }

    pub fn backspace(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.col = WIDTH - 1;
        } else {
            self.hw_cursor_update();
            return;
        }
        let blank = ((self.color as u16) << 8) | (b' ' as u16);
        unsafe {
            self.write_cell(self.row, self.col, blank);
        }
        self.hw_cursor_update();
    }
}

/// Safe because we use SpinLock to ensure exclusive access.
unsafe impl Send for VgaTextConsole {}

impl Console for VgaTextConsole {
    fn clear_screen(&mut self) {
        let blank = ((self.color as u16) << 8) | b' ' as u16;
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                unsafe { self.write_cell(row, col, blank) };
            }
        }
        self.col = 0;
        self.row = 0;
        self.hw_cursor_update();
    }

    fn set_color(&mut self, fg: u8, bg: u8) {
        self.color = color_code(fg, bg);
    }

    fn write_byte(&mut self, b: u8) {
        match b {
            b'\n' => self.newline(),
            ch => {
                if self.col >= WIDTH {
                    self.newline();
                }
                unsafe {
                    self.write_cell(self.row, self.col, self.pack(ch));
                }
                self.col += 1;
            }
        }
        self.hw_cursor_update();
    }
}

pub static CONSOLE: SpinLock<VgaTextConsole> = SpinLock::new(VgaTextConsole::new());

pub fn try_with_console<F: FnOnce(&mut VgaTextConsole)>(f: F) {
    if let Some(mut g) = CONSOLE.try_lock() {
        f(&mut *g);
    } else {
        // drop silently
    }
}
