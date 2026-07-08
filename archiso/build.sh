#!/bin/bash
# Automate compilation of Rust TUI installer and building the custom Archiso

set -e

# Lokasi direktori kerja
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALLER_DIR="$WORKSPACE_DIR/installer"
BUILD_DIR="$SCRIPT_DIR/build_workspace"
OUT_DIR="$SCRIPT_DIR/out"

echo "=================================================="
echo "   MEMULAI PROSES BUILD ARCH GAMING LINUX ISO     "
echo "=================================================="

# 1. Periksa apakah paket archiso terpasang di host
if ! command -v mkarchiso &> /dev/null; then
    echo "Error: Paket 'archiso' belum terpasang di sistem host Anda."
    echo "Silakan pasang terlebih dahulu dengan menjalankan: sudo pacman -S archiso"
    exit 1
fi

# 2. Kompilasi Installer Rust TUI ke mode Release
echo "-> Mengompilasi Installer Rust TUI (Mode Release)..."
cargo build --manifest-path "$INSTALLER_DIR/Cargo.toml" --release

# 3. Buat direktori output dan workspace bersih
echo "-> Mempersiapkan direktori kerja..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
mkdir -p "$OUT_DIR"

# 4. Salin profil baseline archiso resmi sebagai template dasar
echo "-> Menyalin profil baseline archiso..."
cp -r /usr/share/archiso/configs/baseline/* "$BUILD_DIR/"

# 5. Salin paket manifest kustom (packages.x86_64) ke workspace
echo "-> Menyalin konfigurasi paket..."
cp "$SCRIPT_DIR/packages.x86_64" "$BUILD_DIR/packages.x86_64"

# Aktifkan multilib di pacman.conf archiso
echo "-> Mengaktifkan multilib di pacman.conf archiso..."
cat <<EOF >> "$BUILD_DIR/pacman.conf"

[multilib]
Include = /etc/pacman.d/mirrorlist
EOF

# 6. Buat struktur folder airootfs di workspace
echo "-> Menyalin custom airootfs overlay..."
mkdir -p "$BUILD_DIR/airootfs/usr/local/bin"
mkdir -p "$BUILD_DIR/airootfs/etc/systemd/system/getty@tty1.service.d"
mkdir -p "$BUILD_DIR/airootfs/root"

# Salin biner hasil kompilasi Rust installer
cp "$INSTALLER_DIR/target/release/gaming-installer" "$BUILD_DIR/airootfs/usr/local/bin/gaming-installer"

# Salin skrip bash install_base.sh
cp "$SCRIPT_DIR/airootfs/usr/local/bin/install_base.sh" "$BUILD_DIR/airootfs/usr/local/bin/install_base.sh"
chmod +x "$BUILD_DIR/airootfs/usr/local/bin/install_base.sh"

# Salin konfigurasi autologin TTY1
cp "$SCRIPT_DIR/airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf" \
   "$BUILD_DIR/airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf"

# Salin skrip start .bash_profile
cp "$SCRIPT_DIR/airootfs/root/.bash_profile" "$BUILD_DIR/airootfs/root/.bash_profile"

# Salin paket kernel kustom prebuilt jika tersedia di workspace
KERNEL_PKGS_DIR="$WORKSPACE_DIR/kernel_packages"
if [ -d "$KERNEL_PKGS_DIR" ] && ls "$KERNEL_PKGS_DIR"/*.pkg.tar.zst &>/dev/null; then
    echo "-> Menyalin paket kernel kustom prebuilt ke dalam ISO overlay..."
    mkdir -p "$BUILD_DIR/airootfs/opt/kernel-packages"
    cp "$KERNEL_PKGS_DIR"/*.pkg.tar.zst "$BUILD_DIR/airootfs/opt/kernel-packages/"
fi

# 7. Eksekusi mkarchiso untuk mem-build ISO final
echo "-> Menjalankan mkarchiso untuk mengemas sistem (memerlukan hak root)..."
echo "Menjalankan perintah: sudo mkarchiso -v -w \"$BUILD_DIR/work\" -o \"$OUT_DIR\" \"$BUILD_DIR\""
echo "Silakan masukkan password sudo jika diminta."

sudo mkarchiso -v -w "$BUILD_DIR/work" -o "$OUT_DIR" "$BUILD_DIR"

echo "=================================================="
echo "    BUILD SELESAI DENGAN SUKSES!                   "
echo "    ISO: $OUT_DIR/archlinux-*.iso                 "
echo "=================================================="
