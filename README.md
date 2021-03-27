# Project MoonDust

A simple kernel aimed at learning Operating Systems written in Rust.

## Features

- Multi processor Kernel (unstable/WIP)
- CPU Local storage using `#[thread_local]` and ELF sections
- Cooperative scheduling using rust's async patterns (Uses [executor](https://github.com/smol-rs/async-executor))
- Single stack per CPU because of fully async kernel code

## More

- Pure Rust kernel with [bootboot](https://gitlab.com/bztsrc/bootboot) bootloader
- Requires rust nightly for building. Builds on MacOS, Linux and WSL
- Full GDB debugging via VSCode. Contains settings for VSCode out of box. Requires some plugins.
- Run `make` to build and `make efi` to run qemu
