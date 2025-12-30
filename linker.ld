ENTRY(_start)

SECTIONS {
    . = 0x400000;

    /* Forçar _start a ser o primeiro símbolo */
    .text : ALIGN(4096) {
        KEEP(*(.text._start))
        KEEP(*(.text.start))
        *(.text .text.*)
    }

    .rodata : ALIGN(4096) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4096) {
        *(.data .data.*)
    }

    .bss : ALIGN(4096) {
        *(.bss .bss.*)
        *(COMMON)
    }
}
