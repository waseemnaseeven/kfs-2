; =============================================================================
; Bootloader entry stub for a Multiboot-compliant kernel
; =============================================================================
; This assembly file provides:
;   - A valid Multiboot header (recognized by GRUB and other multiboot loaders)
;   - A simple stack setup (16 KiB)
;   - A call into the Rust kernel entry point (_start_kernel)
;     using the cdecl ABI with (magic, mbi) taken from EAX/EBX
;   - A safe infinite halt loop after returning
; =============================================================================

[BITS 32]

; -----------------------------------------------------------------------------
; Multiboot header constants
; -----------------------------------------------------------------------------
MBALIGN   equ 1 << 0                  ; Align modules on page boundaries
MEMINFO   equ 1 << 1                  ; Request memory map from bootloader
MBFLAGS   equ MBALIGN | MEMINFO       ; Combine flags
MAGIC     equ 0x1BADB002              ; Required "magic number"
CHECKSUM  equ -(MAGIC + MBFLAGS)      ; Ensure (magic + flags + checksum) == 0

; -----------------------------------------------------------------------------
; Multiboot header (must be in the first 8 KiB of the kernel binary)
; -----------------------------------------------------------------------------
section .multiboot
align 4
    dd MAGIC                          ; Magic number for multiboot compliance
    dd MBFLAGS                        ; Flags requested from the bootloader
    dd CHECKSUM                       ; Ensures validity of header

; -----------------------------------------------------------------------------
; Uninitialized data section (.bss)
; Reserve 16 KiB for the initial stack (simple static stack)
; -----------------------------------------------------------------------------
section .bss
global stack_bottom
global stack_top
align 16
stack_bottom:                         ; Bottom of stack (lowest address)
    resb 16384                        ; 16 KiB reserved for stack
stack_top:                            ; Label for top of stack (highest address)

; -----------------------------------------------------------------------------
; Kernel entry point
; GRUB jumps here after loading the kernel into memory.
; -----------------------------------------------------------------------------
section .text
global _start
_start:
    ; Initialize stack pointer (ESP) to the top of our reserved stack
    mov esp, stack_top

    ; GRUB provides:
    ;   EAX = 0x2BADB002 (Multiboot magic)
    ;   EBX = pointer to multiboot_info structure
    ;
    ; Pass them to Rust (cdecl): push last arg first
    extern _start_kernel
    push ebx            ; mbi address (2nd argument)
    push eax            ; magic value (1st argument)
    call _start_kernel
    add esp, 8          ; clean up the stack (cdecl)

    ; If the kernel ever returns, disable interrupts and halt forever
    cli

.hang:
    hlt                                ; Halt CPU until next interrupt
    jmp .hang                          ; Infinite loop

; -----------------------------------------------------------------------------
; Mark stack as non-executable (for security, used by some linkers)
; -----------------------------------------------------------------------------
section .note.GNU-stack
