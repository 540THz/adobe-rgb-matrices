Push-Location $PSScriptRoot

# See https://github.com/johnthagen/min-sized-rust
# for information on how to reduce the binary size in Rust

# ------------------- #### Prerequisites #### -------------------
# 1. Install Rust
#    Windows x64 (64-bit): https://win.rustup.rs/x86_64 (recommended)
#    Windows x86 (32-bit): https://win.rustup.rs/i686
#
# 2. Install Git
#    https://git-scm.com/downloads/win
#
# 3. Install nightly toolchain and libstd’s source code for nightly toolchain
#    > rustup toolchain install nightly
#    > rustup component add rust-src --toolchain nightly
#    See https://doc.rust-lang.org/cargo/reference/unstable.html#requirements
#
# 4. For *-windows-msvc target, install Visual C++ Build Tools or Visual Studio
#    https://visualstudio.microsoft.com/visual-cpp-build-tools/
#    https://visualstudio.microsoft.com/downloads/
#
# 5. For *-windows-gnu / *-windows-gnullvm target, install MSYS2
#    https://www.msys2.org/docs/installer/
#    and REPLACE D:\msys64 in $msys64_dir of line 68 with the ACTUAL INSTALL LOCATION of MSYS2.
#
#    Run the following commands on MSYS2 shell.
#    $ pacman -Syu
#    $ pacman -Su
#
#    5-1) For *-windows-gnu target, run
#    $ pacman -S --needed base-devel mingw-w64-i686-toolchain
#    $ pacman -S --needed base-devel mingw-w64-x86_64-toolchain
#    on MSYS2 shell.
#
#    5-2) For *-windows-gnullvm target, run
#    $ pacman -S --needed base-devel mingw-w64-clang-i686-toolchain
#    $ pacman -S --needed base-devel mingw-w64-clang-x86_64-toolchain
#    on MSYS2 shell.
#
#    See https://doc.rust-lang.org/rustc/platform-support/pc-windows-gnullvm.html 
#        https://www.msys2.org/docs/environments/
#        https://www.msys2.org/docs/package-management/
#
# ---------------------------------------------------------------

$keys = @(
    # Add the keys for the targets you want to build here
    "i686", "x86_64"
    # "i686g", "x86_64g"
    # "i686gl", "x86_64gl"
)

$targets = @{
    "i686"     = "i686-win7-windows-msvc"
    "x86_64"   = "x86_64-win7-windows-msvc"
    "i686g"    = "i686-pc-windows-gnu"
    "x86_64g"  = "x86_64-pc-windows-gnu"
    "i686gl"   = "i686-pc-windows-gnullvm"
    "x86_64gl" = "x86_64-pc-windows-gnullvm"
}

$targets_to_add = @{
    "i686"     = "i686-pc-windows-msvc"
    "x86_64"   = "x86_64-pc-windows-msvc"
}

$msys64_dir = "D:\msys64"
$toochain_dirs = @{
    "i686g"    = "$msys64_dir\mingw32\bin"
    "x86_64g"  = "$msys64_dir\mingw64\bin"
    "i686gl"   = "$msys64_dir\clang32\bin"
    "x86_64gl" = "$msys64_dir\clang64\bin"
}

$is_static_crt = $true
$static_crt = @(
    "-C", "target-feature=+crt-static"
    "-C", "link-args=/Brepro"
)
$static_vcruntime = @(
    "-C", "target-feature=+crt-static"
    "-C", "link-args=/DEFAULTLIB:ucrt.lib"
    "-C", "link-args=/NODEFAULTLIB:libucrt.lib"
    "-C", "link-args=/Brepro"
)
$additional_rustflags = @{
    "i686"     = if ($is_static_crt) {$static_crt} else {$static_vcruntime}
    "x86_64"   = if ($is_static_crt) {$static_crt} else {$static_vcruntime}
}


# GET NAME OF BINARY EXECUTABLE
$path_main_rs = Join-Path $PWD "src/main.rs"
$name = foreach ($package in (cargo metadata --format-version 1 | ConvertFrom-Json).packages) {
    foreach ($target in $package.targets) {
        if ((Resolve-Path $target.src_path).Path -eq $path_main_rs) {
            $target.name
        }
    }
}
if ($name -is [string]) {
    "$name"
} else {
    [Console]::Error.WriteLine("error: couldn't get name of binary executable")
    Pop-Location
    exit 1
}

# ADD NECESSARY TARGETS (if it has not been added)
foreach ($k in $keys) {
    $target = if ($targets_to_add[$k]) {$targets_to_add[$k]} else {$targets[$k]}
    rustup target add $target --toolchain nightly
}

# BUILD BINARY EXECUTABLE FOR EACH TARGET
$null = mkdir out -ErrorAction SilentlyContinue
$rustflags = $env:RUSTFLAGS
$path = $env:Path
$ret = 0
foreach ($k in $keys) {
    $env:RUSTFLAGS = "-Zlocation-detail=none -Zfmt-debug=none $($additional_rustflags[$k])".TrimEnd()
    $env:Path = "$path;$($toochain_dirs[$k])".TrimEnd(";")
    cargo +nightly build -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size,panic_immediate_abort --target $targets[$k] --release
    if (-not $?) {$ret = 1; break}
    Copy-Item "target\$($targets[$k])\release\$name.exe" "out\$name-$k.exe"
    if (-not $?) {$ret = 1; break}
}
$env:Path = $path
$env:RUSTFLAGS = $rustflags

Pop-Location
exit $ret
