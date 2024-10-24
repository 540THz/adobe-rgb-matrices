#!/bin/sh
cd "${0%/*}"

# See https://github.com/johnthagen/min-sized-rust
# for information on how to reduce the binary size in Rust

# ------------------- #### Prerequisites #### -------------------
# 1. Install Rust
#    $ curl https://sh.rustup.rs -sSf | sh -s
#    
# 2. Install Git
#    $ sudo apt install git    # on Ubuntu
#
# 3. Install nightly toolchain and libstd’s source code for nightly toolchain
#    $ rustup toolchain install nightly
#    $ rustup component add rust-src --toolchain nightly
#    See https://doc.rust-lang.org/cargo/reference/unstable.html#requirements
#
# 4. Install lld
#    $ sudo apt install lld    # on Ubuntu
#
# 5. Add the following lines to ~/.cargo/config.toml
#    [target.armv7-unknown-linux-musleabihf]
#    linker = "ld.lld"
#    
#    [target.aarch64-unknown-linux-musl]
#    linker = "ld.lld"
#    
#    [target.i686-unknown-linux-musl]
#    linker = "ld.lld"
#    
#    [target.x86_64-unknown-linux-musl]
#    linker = "ld.lld"
#
# ---------------------------------------------------------------

targets="
    armv7-unknown-linux-musleabihf|armv7hf
    aarch64-unknown-linux-musl|aarch64
    i686-unknown-linux-musl|i686
    x86_64-unknown-linux-musl|x86_64
"

# GET NAME OF BINARY EXECUTABLE
name=$(cargo metadata --format-version 1 | sed -nE 's!.*"name":"([a-zA-Z0-9_-]+)","src_path":"'"$PWD"'/src/main.rs".*!\1!p')
[ -n "$name" ] && echo "$name" || { echo "error: couldn't get name of binary executable" 1>&2; exit 1; }

# ADD NECESSARY TARGETS (if it has not been added)
for target in $targets; do
    rustup target add ${target%|*} --toolchain nightly
done

# BUILD BINARY EXECUTABLE FOR EACH TARGET
mkdir -p out
for target in $targets; do
    RUSTFLAGS="-Zlocation-detail=none -Zfmt-debug=none" cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size,panic_immediate_abort --target ${target%|*} --release || exit 1
    cp -p "target/${target%|*}/release/$name" "out/$name-${target##*|}" || exit 1
done
