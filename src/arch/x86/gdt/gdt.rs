use core::arch::asm;
use core::ptr;

use crate::println;

/*
    Descriptor limit for a flat 4 GiB segment (20-bit limit replicated with granularity bit).
*/
const LIMIT_4GB: u32 = 0x000F_FFFF;

/*
    Common granularity flags: 4 KiB blocks + 32-bit default operation size.
*/
const GRANULARITY_FLAGS: u8 = 0b1100;

/*
    Physical address where we copy the GDT before loading it.
*/
const GDT_PHYS_ADDR: u32 = 0x0000_0800;

/*
    Selectors/index for the descriptors we build inside the GDT.
*/
pub(crate) const KERNEL_CODE_SELECTOR: u16 = 1 << 3;
pub(crate) const KERNEL_DATA_SELECTOR: u16 = 2 << 3;
pub(crate) const KERNEL_STACK_SELECTOR: u16 = 3 << 3;
pub(crate) const USER_CODE_SELECTOR: u16 = (4 << 3) | 0b11;
pub(crate) const USER_DATA_SELECTOR: u16 = (5 << 3) | 0b11;
pub(crate) const USER_STACK_SELECTOR: u16 = (6 << 3) | 0b11;

extern "C" {
    static stack_bottom: u8;
    static stack_top: u8;
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct GdtEntry(u64);

/*
   63                                32 31                               0
   ┌───────────────────────────────────┬───────────────────────────────────┐
   │ Base[31:24] | Flags | Limit[19:16]| Access | Base[23:16] | Base[15:0] │
   ├──────────────┴────────────┴────────┴─────────┴─────────────┴──────────┤
   │                Limit[15:0]                                            │
   └───────────────────────────────────────────────────────────────────────┘
*/
impl GdtEntry {
    const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        let mut value = 0u64;
        value |= (limit & 0xFFFF) as u64;
        value |= ((base & 0xFFFF) as u64) << 16;
        value |= (((base >> 16) & 0xFF) as u64) << 32;
        value |= (access as u64) << 40;
        value |= (((limit >> 16) & 0xF) as u64) << 48;
        value |= (flags as u64) << 52;
        value |= (((base >> 24) & 0xFF) as u64) << 56;
        GdtEntry(value)
    }
}

const fn privilege_mask(ring: u8) -> u8 {
    (ring & 0b11) << 5
}

const fn code_segment(ring: u8) -> GdtEntry {
    GdtEntry::new(0, LIMIT_4GB, 0x9A | privilege_mask(ring), GRANULARITY_FLAGS)
}

const fn data_segment(ring: u8) -> GdtEntry {
    GdtEntry::new(0, LIMIT_4GB, 0x92 | privilege_mask(ring), GRANULARITY_FLAGS)
}

const fn stack_segment(ring: u8) -> GdtEntry {
    // Stack uses a dedicated descriptor so that we can change privilege level bits independently later.
    GdtEntry::new(0, LIMIT_4GB, 0x96 | privilege_mask(ring), GRANULARITY_FLAGS)
}

/*
    Template table that we copy to physical 0x800 before loading it.
*/
#[used]
static GDT_TEMPLATE: [GdtEntry; 7] = [
    GdtEntry(0),
    code_segment(0),
    data_segment(0),
    stack_segment(0),
    code_segment(3),
    data_segment(3),
    stack_segment(3),
];

/*
    GDTR-compatible pointer (limit + base) passed to the `lgdt` instruction.
*/
#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u32,
}

#[repr(C, packed)]
struct FarPointer {
    offset: u32,
    selector: u16,
}

pub fn init_with_entry(entry: extern "C" fn() -> !) -> ! {
    unsafe { init_gdt_and_jump(entry) }
}

unsafe fn init_gdt_and_jump(entry: extern "C" fn() -> !) -> ! {
    /*
        Copy the 7 entries towards 0x800
    */
    let gdt_destination = GDT_PHYS_ADDR as *mut GdtEntry;
    ptr::copy_nonoverlapping(GDT_TEMPLATE.as_ptr(), gdt_destination, GDT_TEMPLATE.len());

    /*
        Building a GDTR pointer
    */
    let gdt_ptr = DescriptorTablePointer {
        limit: (core::mem::size_of::<[GdtEntry; 7]>() - 1) as u16,
        base: GDT_PHYS_ADDR,
    };

    /*
        Far pointer used to reload CS and validate the new GDT.
    */
    let entry_ptr = FarPointer {
        offset: entry as u32,
        selector: KERNEL_CODE_SELECTOR,
    };

    load_gdt_and_segments(&gdt_ptr, &entry_ptr);
}

unsafe fn load_gdt_and_segments(
    gdt_ptr: &DescriptorTablePointer,
    entry: &FarPointer,
) -> ! {
    asm!(
        "cli",
        "lgdt [{gdt_ptr}]",
        "mov ax, {data_sel}",
        "mov ds, ax",
        "mov es, ax",
        "mov fs, ax",
        "mov gs, ax",
        "mov ax, {stack_sel}",
        "mov ss, ax",
        "lea esp, [{stack_ptr}]",
        "ljmp [{entry}]",
        gdt_ptr = in(reg) gdt_ptr,
        entry = in(reg) entry,
        data_sel = const KERNEL_DATA_SELECTOR,
        stack_sel = const KERNEL_STACK_SELECTOR,
        stack_ptr = sym stack_top,
        options(noreturn),
    );
}

pub fn print_stack() {
    let (bottom, top) = unsafe { stack_bounds() };
    let esp = current_stack_pointer();

    println!(
        "Kernel stack range: {:#010X} - {:#010X}",
        bottom, top
    );
    println!("Current ESP: {:#010X}", esp);

    if esp < bottom || esp >= top {
        println!("ESP is outside of the kernel stack!");
        return;
    }

    let mut addr = esp;
    let mut count = 0;
    const STACK_DUMP_ENTRIES: usize = 8;
    while addr < top && count < STACK_DUMP_ENTRIES {
        let value = unsafe { core::ptr::read(addr as *const u32) };
        println!("{:#010X}: {:#010X}", addr, value);
        addr += 4;
        count += 1;
    }
}

/*
    Translate the assembly labels into usable Rust addresses
*/
unsafe fn stack_bounds() -> (u32, u32) {
    (
        &stack_bottom as *const u8 as u32,
        &stack_top as *const u8 as u32,
    )
}

fn current_stack_pointer() -> u32 {
    let esp: u32;
    unsafe {
        asm!("mov {0}, esp", out(reg) esp);
    }
    esp
}
