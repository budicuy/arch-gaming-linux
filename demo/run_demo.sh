#!/bin/bash
# Script untuk menjalankan demo simulasi Rust TUI installer secara lokal

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALLER_DIR="$WORKSPACE_DIR/installer"

echo "=================================================="
echo "   MENJALANKAN DEMO SIMULASI ARCH GAMING INSTALLER"
echo "=================================================="
echo "-> Skrip ini akan mengompilasi installer Rust lokal"
echo "   dan menjalankannya dalam mode simulasi/dummy."
echo "-> Mode simulasi akan aktif otomatis karena skrip"
echo "   instalasi riil (/usr/local/bin/install_base.sh)"
echo "   tidak ada di mesin host Anda."
echo "=================================================="
sleep 1.5

# 1. Kompilasi aplikasi Rust
echo "-> Memeriksa dan mengompilasi kode Rust..."
cargo build --manifest-path "$INSTALLER_DIR/Cargo.toml"

# 2. Jalankan biner installer
echo "-> Menjalankan biner installer..."
echo "Tips Navigasi:"
echo " - [Enter] / [Tab] : Lanjut / Pindah panel"
echo " - [Esc] / [Backspace] : Kembali ke langkah sebelumnya"
echo " - [Space] : Toggle pilihan checkbox (pada menu paket gaming)"
echo " - [Panah Up/Down] : Navigasi pilihan list"
echo " - [Ctrl + C] : Keluar dari demo"
echo "--------------------------------------------------"
echo "Memulai dalam 2 detik..."
sleep 2

# Jalankan biner
"$INSTALLER_DIR/target/debug/gaming-installer"
