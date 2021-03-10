# Project MoonDust

A simple kernel aimed at learning Operating Systems written in Rust.

## Features

- Single processor kernel, preparing for multi processor
- Text based UI using [tui-rs](https://github.com/vinaychandra/tui-rs)
- CPU Local storage using `#[thread_local]` and ELF sections
- Cooperative scheduling using rust's async patterns (Uses [executor](https://github.com/smol-rs/async-executor))

## More

- Pure Rust kernel with [bootboot](https://gitlab.com/bztsrc/bootboot) bootloader
- Requires rust nightly for building. Builds on MacOS, Linux and WSL
- Full GDB debugging via VSCode. Contains settings for VSCode out of box. Requires some plugins.
- Run `make` to build and `make efi` to run qemu
