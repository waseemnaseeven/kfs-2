use core::arch::asm;

/// Send an 8-bit value to an I/O port.
///
/// # Safety
/// Directly accesses hardware. The caller must ensure the port address is valid for the current hardware.
pub unsafe fn outb(port: u16, val: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nostack, preserves_flags)
    );
}

/// Read an 8-bit value from an I/O port.
///
/// # Safety
/// Directly accesses hardware. Reading from the wrong port can cause undefined behavior.
pub unsafe fn inb(port: u16) -> u8 {
    let mut v: u8;
    asm!(
        "in al, dx",
        out("al") v,
        in("dx") port,
        options(nostack, preserves_flags)
    );
    v
}
