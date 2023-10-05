Integration tests
=================

This directory contains integration tests written in Objective-C. They're compiled to an ARMv6 Mach-O binary and packaged into a bundle (`TestApp.app`) so that they can be run in the emulator like a normal iPhone OS app. The code in `integration.rs` lets them be run by `cargo test` (which also runs unit tests written in Rust).

Building
--------

### Setup
To set up integration tests, a copy of `clang` and Apple's `ld` (either native or ported) is required. 

Both the compiler and linker and searched for in the following order:
1. Environment variables `TOUCHHLE_COMPILER` and `TOUCHHLE_LINKER` respectively. (These should point directly to the program)
2. The pathes `{touchHLE repository}/tests/llvm/bin` and `{touchHLE repository}/tests/cctools/bin`.
3. The programs `clang` and `ld`. (The latter is only searched for on MacOS, use one of the above methods for other platforms)

After the compiler and linker are setup, you can just run `cargo test`
#### Linker Setup (required for non-MacOS)
To link oull need to use an [unofficial port](https://github.com/tpoechtrager/cctools-port/tree/master). Compile according to the instructions provided in the repo (you'll probably want to set the `--prefix` (output directory) flag in `./configure.sh`, to avoid having Apple's `ld` override your system linker) and set the environment variable `TOUCHHLE_LINKER=/path/to/build-output/bin/ld` before running `cargo test`.

### Particulars

- Binaries linked with new versions of ld have a larger null page size (16kB instead of 4kB).
