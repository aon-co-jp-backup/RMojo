#!/usr/bin/env bash
# Build rmojo for the Android TV x86 emulator (i686-linux-android) inside WSL using the Windows SDK's NDK.
# usage (PowerShell): wsl -d Ubuntu -- bash /mnt/f/RMojo/scripts/android-i686-wsl.sh
set -e
export PATH="$HOME/.cargo/bin:/usr/bin:/bin"
NDK_WIN="/mnt/c/Users/noruk/AppData/Local/Android/Sdk/ndk"
NDK="$NDK_WIN/$(ls "$NDK_WIN" | tail -1)"
# the Windows NDK holds .exe tools; use the Linux host tools if present, else fall back to cargo-ndk
cd "$(dirname "$0")/.."
which cargo || { echo "no cargo in WSL"; exit 1; }
rustup target add i686-linux-android 2>&1 | tail -1
ls "$NDK/toolchains/llvm/prebuilt" || true
which cargo-ndk || cargo install cargo-ndk --locked 2>&1 | tail -1
ANDROID_NDK_HOME="$NDK" cargo ndk -t x86 build --release 2>&1 | tail -5
ls -la target/i686-linux-android/release/rmojo
