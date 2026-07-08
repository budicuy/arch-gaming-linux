#!/bin/bash
# Script to compile linux-tkg-zen into a binary package inside GitHub Actions
# Must be executed as root (it will handle running makepkg as a non-root user)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PKG_OUT_DIR="$WORKSPACE_DIR/kernel_packages"

echo "=================================================="
echo "   COMPILING LINUX-TKG-ZEN KERNEL PACKAGE         "
echo "=================================================="

# 1. Buat folder output paket
mkdir -p "$PKG_OUT_DIR"

# Periksa apakah kernel sudah terkompilasi sebelumnya (dari cache)
# linux-tkg menghasilkan paket bernama: linux-tkg-zen-*.pkg.tar.zst
if ls "$PKG_OUT_DIR"/linux-tkg-zen-*.pkg.tar.zst &>/dev/null; then
    echo "-> Menemukan paket kernel prebuilt di cache. Melewati kompilasi."
    exit 0
fi

# 2. Buat user non-root 'builder' untuk makepkg jika belum ada
if ! id -u builder >/dev/null 2>&1; then
    echo "-> Membuat user 'builder' untuk kebutuhan makepkg..."
    useradd -m builder
    echo "builder ALL=(ALL) NOPASSWD: ALL" >> /etc/sudoers
fi

# 3. Setup build directory di /home/builder/build
BUILD_DIR="/home/builder/build"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Salin customization.cfg ke folder build
cp "$SCRIPT_DIR/customization.cfg" "$BUILD_DIR/customization.cfg"
chown -R builder:builder "$BUILD_DIR"

# 4. Jalankan kloning dan kompilasi sebagai user builder
echo "-> Memulai kloning & kompilasi linux-tkg (sebagai user 'builder')..."
sudo -u builder bash -c '
    set -e
    BUILD_DIR="/home/builder/build"
    git clone https://github.com/Frogging-Family/linux-tkg.git "$BUILD_DIR/linux-tkg"
    cp "$BUILD_DIR/customization.cfg" "$BUILD_DIR/linux-tkg/"
    cd "$BUILD_DIR/linux-tkg"
    
    # Jalankan makepkg untuk memicu pembuatan paket Arch
    # -s: pasang dependensi pacman otomatis, -r: hapus dep setelah build, --noconfirm: setujui semua prompt
    makepkg --noconfirm -s -r
'

# 5. Ambil file paket hasil build (.pkg.tar.zst) ke folder output workspace
echo "-> Menyalin file .pkg.tar.zst hasil build ke output..."
find "$BUILD_DIR/linux-tkg" -name "linux-tkg-zen-*.pkg.tar.zst" -exec cp {} "$PKG_OUT_DIR/" \;
find "$BUILD_DIR/linux-tkg" -name "linux-tkg-zen-headers-*.pkg.tar.zst" -exec cp {} "$PKG_OUT_DIR/" \;

echo "-> Kompilasi selesai. Paket kernel berhasil disimpan di: $PKG_OUT_DIR/"
ls -la "$PKG_OUT_DIR/"
