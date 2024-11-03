Push-Location $PSScriptRoot

# For information on how to reduce the binary size in Rust, see [1].
# For using a specific version of nightly toolchain (e.g., nightly-2024-10-31), see [2].
# For `CARGO_PROFILE_RELEASE_DEBUG` and `CARGO_PROFILE_RELEASE_STRIP`, see [3] and [4].
# [1] https://github.com/johnthagen/min-sized-rust
# [2] https://rust-lang.github.io/rustup/overrides.html
# [3] https://doc.rust-lang.org/cargo/reference/config.html#profilenamedebug
# [4] https://doc.rust-lang.org/cargo/reference/config.html#profilenamestrip

# ------------------- #### Prerequisites #### -------------------
# 1. Install Rust
#    Windows x64 (64-bit): https://win.rustup.rs/x86_64 (recommended)
#    Windows x86 (32-bit): https://win.rustup.rs/i686
#
# 2. Install Git
#    https://git-scm.com/downloads/win
#
# 3. For *-windows-msvc target, install Visual Studio (any edition)
#    or Visual C++ Build Tools (aka Visual Studio Build Tools).
#    https://visualstudio.microsoft.com/downloads/
#    https://visualstudio.microsoft.com/visual-cpp-build-tools/
#    When installing, choose the "Desktop development with C++" workload.
#    See https://rust-lang.github.io/rustup/installation/windows-msvc.html
#    for minimal installation.
#
# 4. For *-windows-gnu / *-windows-gnullvm target,
#    4-1) Install MSYS2
#    https://github.com/msys2/msys2-installer/releases
#
#    4-2) IMPORTANT: Replace "D:\msys64" in $msys64_dir on line 72 of this file
#                    with the ACTUAL INSTALLED LOCATION of MSYS2
#
#    4-3) Run the following commands on MSYS2 shell
#    $ pacman -Syu   # When it's done, hit 'y <enter>' and reopen MSYS2 shell to continue
#    $ pacman -Su
#
#    4-4) For *-windows-gnu target, run these commands on MSYS2 shell
#    $ pacman -S --needed base-devel mingw-w64-i686-toolchain
#    $ pacman -S --needed base-devel mingw-w64-x86_64-toolchain
#
#    4-5) For *-windows-gnullvm target, run these commands on MSYS2 shell
#    $ pacman -S --needed base-devel mingw-w64-clang-i686-toolchain
#    $ pacman -S --needed base-devel mingw-w64-clang-x86_64-toolchain
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
$msys64_bin_dirs = @{
    "i686g"    = "$msys64_dir\mingw32\bin"
    "x86_64g"  = "$msys64_dir\mingw64\bin"
    "i686gl"   = "$msys64_dir\clang32\bin"
    "x86_64gl" = "$msys64_dir\clang64\bin"
}

$is_static_vcruntime = $true
$is_static_ucrt      = $true
$tmp = "-C", "link-args=/Brepro"
if ($is_static_vcruntime) {
    $tmp += "-C", "target-feature=+crt-static"
    if (-not $is_static_ucrt) {
        $tmp += "-C", "link-args=/DEFAULTLIB:ucrt.lib",
                "-C", "link-args=/NODEFAULTLIB:libucrt.lib"
    }
}
$additional_rustflags = @{
    "i686"     = $tmp
    "x86_64"   = $tmp
}

# GET NAME OF BINARY EXECUTABLE
$path_main_rs = Join-Path $PWD "src/main.rs"
$name = foreach ($target in (cargo read-manifest | ConvertFrom-Json).targets) {
    if ((Resolve-Path $target.src_path).Path -eq $path_main_rs) {
        $target.name
    }
}
if ($name -is [string]) {
    "name: $name"
} else {
    [Console]::Error.WriteLine("error: couldn't get name of binary executable")
    Pop-Location
    exit 1
}

# ADD NECESSARY TARGETS (if it has not been added)
foreach ($k in $keys) {
    $target = if ($targets_to_add[$k]) {$targets_to_add[$k]} else {$targets[$k]}
    rustup target add $target
}

# BUILD BINARY EXECUTABLE FOR EACH TARGET
$null = mkdir out -ErrorAction SilentlyContinue
$rustflags = $env:RUSTFLAGS
$path = $env:Path
$debug = $env:CARGO_PROFILE_RELEASE_DEBUG
$strip = $env:CARGO_PROFILE_RELEASE_STRIP
$ret = 0
foreach ($k in $keys) {
    $env:RUSTFLAGS = "-Z location-detail=none -Z fmt-debug=none $($additional_rustflags[$k])".TrimEnd()
    $env:Path = "$path;$($msys64_bin_dirs[$k])".TrimEnd(";")
    if ($targets[$k].EndsWith("-msvc")) {
        # Override 'debug' and 'strip' in [profile.release] in Cargo.toml
        $env:CARGO_PROFILE_RELEASE_DEBUG = "true"
        $env:CARGO_PROFILE_RELEASE_STRIP = "false"
    } else {
        $env:CARGO_PROFILE_RELEASE_STRIP = $strip
        $env:CARGO_PROFILE_RELEASE_DEBUG = $debug
    }
    cargo build -Z build-std=std,panic_abort -Z build-std-features=optimize_for_size,panic_immediate_abort --target $targets[$k] --release
    if (-not $?) {$ret = 1; break}
    # Copy-Item "target\$($targets[$k])\release\$name.exe" "out\$name-$k.exe"
    $s = Get-Item "target\$($targets[$k])\release\$name.exe"
    Copy-Item $s "out\$name-$k.exe"
    if (-not $?) {$ret = 1; break}
    # Preserve original CreationTime and LastAccessTime
    Get-Item "out\$name-$k.exe" | ForEach-Object {$_.CreationTime = $s.CreationTime; $_.LastAccessTime = $s.LastAccessTime}
}
foreach ($k in $keys) {
    if (-not $targets[$k].EndsWith("-msvc")) {continue}
    $null = mkdir "out\$k" -ErrorAction SilentlyContinue
    # Copy-Item "target\$($targets[$k])\release\$name.exe", "target\$($targets[$k])\release\$name.pdb" "out\$k\"
    $ss = Get-Item "target\$($targets[$k])\release\$name.exe", "target\$($targets[$k])\release\$name.pdb"
    Copy-Item $ss "out\$k\"
    if (-not $?) {$ret = 1; break}
    # Preserve original CreationTime and LastAccessTime
    foreach ($s in $ss) {
        Get-Item "out\$k\$($s.Name)" | ForEach-Object {$_.CreationTime = $s.CreationTime; $_.LastAccessTime = $s.LastAccessTime}
    }
}
$env:CARGO_PROFILE_RELEASE_STRIP = $strip
$env:CARGO_PROFILE_RELEASE_DEBUG = $debug
$env:Path = $path
$env:RUSTFLAGS = $rustflags

# # See https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_calculated_properties
# $props = @{n="Creation";e={$_.CreationTime.ToFileTime()};w=18},
#          @{n="LastAccess";e={$_.LastAccessTime.ToFileTime()};w=18},
#          @{n="LastWrite";e={$_.LastWriteTime.ToFileTime()};w=18},
#          @{e="Length";w=9},
#          @{n="SHA1";e={(Get-FileHash $_ -Algorithm SHA1).Hash.ToLower()};w=40},
#          "FullName"
# foreach ($k in $keys) {
#     Get-ChildItem out\*-$k.exe,out\$k\*.exe,out\$k\*.pdb | Format-Table $props
# }

Pop-Location
exit $ret
