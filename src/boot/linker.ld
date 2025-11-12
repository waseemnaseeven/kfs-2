ENTRY(_start)

SECTIONS
{
	/* entry point */
	. = 2M;

	/* Multiboot header */
	.multiboot : ALIGN(4096)
	{
    	KEEP(*(.multiboot))
  	}

	/* Code */
	.text : ALIGN(4096)
  	{
    	*(.text .text.*)
  	}


	/* Read-only data. */
	.rodata : ALIGN(4K)
	{
		*(.rodata .rodata.*)
	}

	/* Read-write data (initialized) */
	.data : ALIGN(4K)
	{
		*(.data .data.*)
	}

	/* Read-write data (uninitialized) and stack */
	.bss : ALIGN(4K)
	{
		*(COMMON)
		*(.bss .bss.*)
	}

	/DISCARD/ : { *(.eh_frame) *(.comment) }
  	.note.GNU-stack : { }
}