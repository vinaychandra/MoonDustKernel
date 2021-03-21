KERNEL_OFFSET = 0xfffffffff0000000;

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

    .got : {
        *(.got*)
        . = ALIGN(4096);
    } :boot

    .tdata : {
        __tdata_start = .;
        *(.tdata*)
        . = ALIGN(4096);
        __tdata_end = .;
    } :boot :tls


    .tbss : {
        __tbss_start = .;
        *(.tbss*)
    } :boot :tls

    /*TBSS has no size. So, we force it to have size here.*/
    . += SIZEOF(.tbss);
    __tbss_end = .;


    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
