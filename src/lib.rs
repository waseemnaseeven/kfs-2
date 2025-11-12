#![no_std]
#![no_main]

use core::panic::PanicInfo;

pub mod arch;
pub mod drivers;
pub mod subsystems;
pub mod sync;

use crate::arch::x86::gdt;
use crate::subsystems::console::vga::vga_color;

#[derive(Copy, Clone)]
struct BootArgs {
    magic: u32,
    mbi_addr: u32,
}

static mut BOOT_ARGS: BootArgs = BootArgs {
    magic: 0,
    mbi_addr: 0,
};

#[no_mangle]
pub extern "C" fn _start_kernel(magic: u32, mbi_addr: u32) -> ! {
    unsafe {
        BOOT_ARGS = BootArgs { magic, mbi_addr };
    }
    gdt::init_with_entry(kernel_entry_post_gdt)
}

extern "C" fn kernel_entry_post_gdt() -> ! {
    let args = unsafe { BOOT_ARGS };
    kernel_main(args.magic, args.mbi_addr)
}

fn kernel_main(magic: u32, mbi_addr: u32) -> ! {
    println!("kfs: boot magic={:#x} mbi={:#x}", magic, mbi_addr);
    crate::subsystems::console::with_color(vga_color::LIGHT_GREEN, vga_color::BLACK, || {
        println!("42");
    });
    gdt::print_stack();

    loop {
        if let Some(ev) = drivers::input::keyboard::poll_event() {
            if let Some(b) = ev.printable_byte() {
                if b == 0x08 {
                    subsystems::console::backspace();
                } else {
                    subsystems::console::write_byte(b);
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("PANIC: {info}");
    loop {}
}
