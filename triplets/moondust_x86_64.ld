KERNEL_OFFSET = 0xfffffffff8000000;

PHDRS
{
  boot PT_LOAD FILEHDR PHDRS;                                /* one single loadable segment */
  tls PT_TLS;
}
SECTIONS
{
    . = KERNEL_OFFSET;
    mmio    = .; . += 0x4000000;
    fb      = .; . += 0x3E00000;
    bootboot    = .; . += 4096;
    environment = .; . += 4096;

    .text . + SIZEOF_HEADERS : AT(ADDR(.text) - . + SIZEOF_HEADERS) {
        KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)   /* code */
		. = ALIGN(4096);
    } :boot

	.rodata : {
        *(.rodata*)
		. = ALIGN(4096);
    } :boot

    .data : {
        *(.data*)
		. = ALIGN(4096);
    } :boot

    .bss : {
        *(.bss*)
        . = ALIGN(4096);
    } :boot

    .tdata : {
        __tdata_start = .;
        *(.tdata*)
		. = ALIGN(4096);
        __tdata_end = .;
        __tbss_start = .;
        *(.tbss*)
        . += 8;
        . = ALIGN(4096);
        __tbss_end = .;
    } :boot :tls

    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
