; Multiboot header for the kernel
MBALIGN  equ  1 << 0            ; Align loaded modules on page boundaries (value = 1)
MEMINFO  equ  1 << 1            ; Provide memory map (value = 2)
MBFLAGS  equ  MBALIGN | MEMINFO  ; Multiboot 'flag' field
MAGIC    equ  0x1BADB002        ; Magic number for the bootloader to find the header
CHECKSUM equ -(MAGIC + MBFLAGS) ; Checksum for Multiboot compliance

; Export stack bounds so the Rust code can inspect them
global stack_bottom
global stack_top

; Multiboot header indicating the program is a kernel
section .multiboot
align 4
    dd MAGIC                    ; Magic number
    dd MBFLAGS                  ; Multiboot flags
    dd CHECKSUM                 ; Checksum, must be zero at the end

; .bss section for uninitialized variables, including the stack
section .bss
align 16
stack_bottom:                     ; Label for the start of the stack space
    resb 16384                    ; Reserve 16 KiB for the kernel_stack
stack_top:                        ; Label for the top of the stack

; .text section contains the executable code of the kernel
section .text
global _start                    ; Declare _start as a global symbol for the bootloader
_start:                         ; Entry point of the kernel
    mov esp, stack_top          ; Initialize stack pointer (ESP) to the top of the stack
    extern _start_kernel         ; Declare external reference to _start_kernel
    call _start_kernel           ; Call the kernel initialization function
    cli                          ; Disable interrupts for critical operations

.hang:                            ; Infinite loop to halt the CPU
    hlt                          ; Halt the CPU until the next interrupt
    jmp .hang                   ; Jump back to the hang label to create an infinite loop

section .note.GNU-stack          ; Note for the linker regarding stack properties
