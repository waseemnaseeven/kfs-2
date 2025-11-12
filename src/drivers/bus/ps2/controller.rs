use crate::arch::x86::port::{inb, outb};

/// PS/2 controller data port (read/write).
/// - Writing: sends a byte to the selected device.
/// - Reading: reads data from the keyboard or mouse.
pub const KBD_DATA: u16 = 0x60;

/// PS/2 controller status register (read-only).
/// Contains bits that indicate input/output buffer state and errors.
pub const KBD_STAT: u16 = 0x64;

/// PS/2 controller command register (write-only).
/// Used to send commands to the controller itself, not to devices.
pub const KBD_CMD: u16 = 0x64;

/// Status flag: Output Buffer Full (data available to read).
/// Set when the controller has data ready in the data port.
const STAT_OBF: u8 = 1 << 0;

/// Status flag: Input Buffer Full (controller busy).
/// Set when the controller is still processing the last command/data write.
const STAT_IBF: u8 = 1 << 1;

pub fn data_available() -> bool {
    unsafe { inb(KBD_STAT) & STAT_OBF != 0 }
}

pub fn read_data() -> u8 {
    unsafe { inb(KBD_DATA) }
}

pub fn write_cmd(cmd: u8) {
    unsafe {
        while inb(KBD_STAT) & STAT_IBF != 0 {
            core::hint::spin_loop()
        }
        outb(KBD_CMD, cmd);
    }
}
