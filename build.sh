#!/bin/sh
cd "${0%/*}"

# For information on how to reduce the binary size in Rust, see [1].
# For using a specific version of nightly toolchain (e.g., nightly-2024-10-31), see [2].
# For `-C linker=rust-lld` in `RUSTFLAGS`, see [3].
# [1] https://github.com/johnthagen/min-sized-rust
# [2] https://rust-lang.github.io/rustup/overrides.html
# [3] https://doc.rust-lang.org/rustc/codegen-options/index.html#linker

# -------- Prerequisites: Install Rust, Git, and GNU Tar --------
#  $ curl https://sh.rustup.rs -sSf | sh -s
#  $ sudo apt install git    # on Ubuntu/Debian
#  $ sudo apt install tar    # on Ubuntu/Debian - if you have removed tar
# ---------------------------------------------------------------

targets="
    armv7-unknown-linux-musleabihf|armv7hf
    aarch64-unknown-linux-musl|aarch64
    i686-unknown-linux-musl|i686
    x86_64-unknown-linux-musl|x86_64
"

# GET NAME OF BINARY EXECUTABLE
name=$(cargo read-manifest | sed -nE 's!.*"name":"([a-zA-Z0-9_-]+)","src_path":"'"$PWD"'/src/main.rs".*!\1!p')
[ -n "$name" ] && echo "name: $name" || { echo "error: couldn't get name of binary executable" 1>&2; exit 1; }

# ADD NECESSARY TARGETS (if it has not been added)
for target in $targets; do
    rustup target add ${target%|*}
done

# BUILD BINARY EXECUTABLE FOR EACH TARGET
mkdir -p out
for target in $targets; do
    RUSTFLAGS="-Z location-detail=none -Z fmt-debug=none -C linker=rust-lld" cargo build -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size,panic_immediate_abort --target ${target%|*} --release || exit 1
    cp "target/${target%|*}/release/$name" "out/$name-${target##*|}" || exit 1
done

# PACK THE BUILT BINARIES INTO A TARBALL
files=""; for target in $targets; do files="$files $name-${target##*|}"; done
ver=$(git describe --tags --abbrev=8)
[ "$ver" = "$(git describe --tags --abbrev=8 --long)" ] && ver="${ver%-*}-${ver##*g}"
sleep 2
tar czfH "out/${PWD##*/}-$ver-linux.tar.gz" posix --mode=0755 --owner=shell:2000 --group=shell:2000 -C out $files
