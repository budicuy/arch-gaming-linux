#!/bin/bash
# High-Performance Arch Linux Gaming System Installer Backend Script
# Di-eksekusi oleh biner TUI installer Rust

set -e

# Setup log file
LOG_FILE="/tmp/install_exec.log"
exec > >(tee -i "$LOG_FILE") 2>&1

echo "=================================================="
echo "   MEMULAI PROSES INSTALASI SYSTEM GAMING BASE   "
echo "=================================================="

# Helper Progress & Status reporter
report_progress() {
    local percent=$1
    local status=$2
    echo "[STATUS] $status"
    echo "[PROGRESS] $percent"
    sleep 0.5
}

# 1. Parsing target partitions
report_progress 5 "Menganalisis disk target..."
if [ -z "$DISK" ]; then
    echo "Error: Variabel DISK kosong!"
    exit 1
fi

# Pendeteksian skema penamaan partisi (NVMe vs SATA/Virtual)
if [[ "$DISK" == nvme* || "$DISK" == mmcblk* ]]; then
    BOOT_PART="/dev/${DISK}p1"
    ROOT_PART="/dev/${DISK}p2"
else
    BOOT_PART="/dev/${DISK}1"
    ROOT_PART="/dev/${DISK}2"
fi

echo "Disk Target: /dev/$DISK"
echo "Partisi Boot: $BOOT_PART"
echo "Partisi Root: $ROOT_PART"

# 2. Skema Partisi Otomatis (jika Auto dipilih)
if [[ "$PART_METHOD" == *"Auto"* ]]; then
    report_progress 10 "Membuat partisi pada /dev/$DISK..."
    # Hapus partition table lama dan buat baru (GPT)
    sgdisk --zap-all "/dev/$DISK"
    
    # Buat partisi EFI (512MB) dan Root (Sisa)
    sgdisk --new=1:0:+512M --typecode=1:ef00 --change-name=1:"EFI Boot" "/dev/$DISK"
    sgdisk --new=2:0:0 --typecode=2:8300 --change-name=2:"Arch Linux Root" "/dev/$DISK"
    
    # Reread partition table
    partprobe "/dev/$DISK"
    sleep 2
    
    # Format EFI ke FAT32
    report_progress 15 "Format EFI system partition..."
    mkfs.vfat -F32 "$BOOT_PART"
    
    # Format Root
    if [[ "$PART_METHOD" == *"Btrfs"* ]]; then
        report_progress 20 "Format BTRFS filesystem pada $ROOT_PART..."
        mkfs.btrfs -f "$ROOT_PART"
        
        report_progress 25 "Membuat subvolume BTRFS..."
        # Mount root sementara untuk buat subvolume
        mkdir -p /mnt_temp
        mount "$ROOT_PART" /mnt_temp
        
        # Buat subvolume standar Arch
        btrfs subvolume create /mnt_temp/@
        btrfs subvolume create /mnt_temp/@home
        btrfs subvolume create /mnt_temp/@cache
        btrfs subvolume create /mnt_temp/@log
        
        umount /mnt_temp
        rmdir /mnt_temp
        
        # Mount subvolume untuk instalasi pacstrap
        report_progress 30 "Melakukan mounting subvolume BTRFS..."
        mkdir -p /mnt
        mount -o noatime,compress=zstd,ssd,space_cache=v2,subvol=@ "$ROOT_PART" /mnt
        mkdir -p /mnt/{home,var/cache,var/log,boot}
        mount -o noatime,compress=zstd,ssd,space_cache=v2,subvol=@home "$ROOT_PART" /mnt/home
        mount -o noatime,compress=zstd,ssd,space_cache=v2,subvol=@cache "$ROOT_PART" /mnt/var/cache
        mount -o noatime,compress=zstd,ssd,space_cache=v2,subvol=@log "$ROOT_PART" /mnt/var/log
    else
        report_progress 20 "Format EXT4 filesystem pada $ROOT_PART..."
        mkfs.ext4 -F "$ROOT_PART"
        
        report_progress 30 "Melakukan mounting filesystem..."
        mkdir -p /mnt
        mount "$ROOT_PART" /mnt
        mkdir -p /mnt/boot
    fi
    # Mount Boot partition
    mount "$BOOT_PART" /mnt/boot
else
    # Untuk Manualcfdisk, asumsikan user sudah mempartisi lewat cfdisk
    # Dan partisi EFI di boot_part, dan Root di root_part
    report_progress 15 "Format partisi manual..."
    mkfs.vfat -F32 "$BOOT_PART"
    
    if [[ "$PART_METHOD" == *"Btrfs"* ]]; then
        mkfs.btrfs -f "$ROOT_PART"
        mkdir -p /mnt
        mount "$ROOT_PART" /mnt
        mkdir -p /mnt/boot
    else
        mkfs.ext4 -F "$ROOT_PART"
        mkdir -p /mnt
        mount "$ROOT_PART" /mnt
        mkdir -p /mnt/boot
    fi
    mount "$BOOT_PART" /mnt/boot
fi

# 3. Pacstrap Base System
report_progress 35 "Menjalankan pacstrap (memakan waktu beberapa menit)..."
pacstrap -K /mnt base base-devel linux-firmware amd-ucode intel-ucode btrfs-progs xfsprogs e2fsprogs dosfstools git networkmanager pipewire pipewire-pulse pipewire-alsa wireplumber grub efibootmgr sudo nano vi

# 4. Generate fstab
report_progress 48 "Membuat file fstab..."
genfstab -U /mnt >> /mnt/etc/fstab

# 5. Konfigurasi Sistem Dasar di Chroot
report_progress 52 "Mengonfigurasi pengaturan regional & sistem..."
arch-chroot /mnt ln -sf /usr/share/zoneinfo/Asia/Jakarta /etc/localtime
arch-chroot /mnt hwclock --systohc

# Set locale
echo "en_US.UTF-8 UTF-8" > /mnt/etc/locale.gen
arch-chroot /mnt locale-gen
echo "LANG=en_US.UTF-8" > /mnt/etc/locale.conf

# Hostname
echo "$HOSTNAME" > /mnt/etc/hostname
echo -e "127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.1.1\t$HOSTNAME.localdomain\t$HOSTNAME" > /mnt/etc/hosts

# Set Password Root
report_progress 56 "Menyetel kata sandi root..."
echo "root:$PASSWORD" | arch-chroot /mnt chpasswd

# Membuat User
report_progress 60 "Membuat pengguna sistem baru..."
arch-chroot /mnt useradd -m -G wheel,audio,video,optical,storage -s /bin/bash "$USERNAME"
echo "$USERNAME:$PASSWORD" | arch-chroot /mnt chpasswd

# Setup Sudoers (allow wheel group)
sed -i 's/# %wheel ALL=(ALL:ALL) ALL/%wheel ALL=(ALL:ALL) ALL/' /mnt/etc/sudoers

# 6. Konfigurasi Repositori Multilib (Penting untuk Steam/Wine)
report_progress 64 "Mengaktifkan repositori multilib..."
cat <<EOF >> /mnt/etc/pacman.conf

[multilib]
Include = /etc/pacman.d/mirrorlist
EOF
arch-chroot /mnt pacman -Sy --noconfirm

# 7. Memasang Desktop / Window Manager
report_progress 68 "Mengonfigurasi Desktop Environment / WM ($DESKTOP)..."
if [[ "$DESKTOP" == *"Hyprland"* ]]; then
    arch-chroot /mnt pacman -S --noconfirm hyprland kitty xdg-desktop-portal-hyprland polkit-kde-agent brightnessctl pavucontrol rofi
elif [[ "$DESKTOP" == *"Sway"* ]]; then
    arch-chroot /mnt pacman -S --noconfirm sway alacritty xdg-desktop-portal-wlr dmenu swaybg swaylock swayidle
elif [[ "$DESKTOP" == *"KdePlasma"* ]]; then
    arch-chroot /mnt pacman -S --noconfirm plasma-desktop sddm konsole dolphin packagekit-qt6
    arch-chroot /mnt systemctl enable sddm
elif [[ "$DESKTOP" == *"Gnome"* ]]; then
    arch-chroot /mnt pacman -S --noconfirm gnome-shell gdm gnome-terminal nautilus
    arch-chroot /mnt systemctl enable gdm
elif [[ "$DESKTOP" == *"Xfce"* ]]; then
    arch-chroot /mnt pacman -S --noconfirm xfce4 xfce4-goodies lightdm lightdm-gtk-greeter
    arch-chroot /mnt systemctl enable lightdm
elif [[ "$DESKTOP" == *"GamescopeOnly"* ]]; then
    arch-chroot /mnt pacman -S --noconfirm gamescope steam
    
    # Buat custom autostart script agar boot langsung ke steam gamescope
    mkdir -p /mnt/home/"$USERNAME"
    cat <<EOF >> /mnt/home/"$USERNAME"/.bash_profile
if [[ -z \$DISPLAY ]] && [[ \$(tty) = /dev/tty1 ]]; then
    # Jalankan Steam dalam Gamescope Micro-Compositor
    gamescope -W 1920 -H 1080 -r 144 -f -- steam -bigpicture
fi
EOF
    chown -R 1000:1000 /mnt/home/"$USERNAME"/.bash_profile
    
    # Set autologin untuk user target pada TTY1
    mkdir -p /mnt/etc/systemd/system/getty@tty1.service.d
    cat <<EOF > /mnt/etc/systemd/system/getty@tty1.service.d/autologin.conf
[Service]
ExecStart=
ExecStart=-/sbin/agetty -o '-p -mx -- \\u' --noclear --autologin $USERNAME %I \$TERM
EOF
fi

# 8. Memasang Paket Gaming & Compatibility Layer
report_progress 74 "Mengunduh & memasang paket-paket gaming..."
GAMING_PKGS=""
if [ "$STEAM" = "true" ]; then GAMING_PKGS="$GAMING_PKGS steam"; fi
if [ "$MANGOHUD" = "true" ]; then GAMING_PKGS="$GAMING_PKGS mangohud lib32-mangohud"; fi
if [ "$GAMEMODE" = "true" ]; then GAMING_PKGS="$GAMING_PKGS gamemode lib32-gamemode"; fi
if [ "$PROTONUP" = "true" ]; then GAMING_PKGS="$GAMING_PKGS protonup-qt"; fi
if [ "$WINE" = "true" ]; then GAMING_PKGS="$GAMING_PKGS wine winetricks lib32-vkd3d vulkan-icd-loader lib32-vulkan-icd-loader"; fi

if [ ! -z "$GAMING_PKGS" ]; then
    arch-chroot /mnt pacman -S --noconfirm $GAMING_PKGS
fi

# 9. Integrasi Tweaks Kernel Gaming (Sysctl)
report_progress 80 "Menginjeksikan optimasi sysctl kernel..."
mkdir -p /mnt/etc/sysctl.d
cat <<EOF > /mnt/etc/sysctl.d/99-gaming.conf
# Sysctl gaming optimization overrides
vm.max_map_count = 2147483642
vm.swappiness = 10
kernel.split_lock_mitigate = 0
net.core.netdev_max_backlog = 16384
net.core.somaxconn = 8192
net.ipv4.tcp_max_syn_backlog = 8192
net.ipv4.tcp_fastopen = 3
net.ipv4.tcp_fin_timeout = 15
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15
net.ipv4.ip_local_port_range = 1024 65535
EOF

# 10. Konfigurasi Kernel Utama (linux-tkg-zen vs official linux-zen)
report_progress 85 "Mengonfigurasi kernel target..."
if ls /opt/kernel-packages/linux-tkg-zen-*.pkg.tar.zst &>/dev/null; then
    echo "-> Menemukan paket kernel kustom linux-tkg-zen prebuilt pada ISO. Memasang..."
    mkdir -p /mnt/tmp/kernel-packages
    cp /opt/kernel-packages/*.pkg.tar.zst /mnt/tmp/kernel-packages/
    arch-chroot /mnt pacman -U --noconfirm /tmp/kernel-packages/linux-tkg-zen-*.pkg.tar.zst /tmp/kernel-packages/linux-tkg-zen-headers-*.pkg.tar.zst
    rm -rf /mnt/tmp/kernel-packages
else
    echo "-> Paket kernel kustom prebuilt tidak ditemukan pada ISO. Fallback memasang official linux-zen..."
    arch-chroot /mnt pacman -S --noconfirm linux-zen linux-zen-headers
fi

# Buat skrip kompilasi otomatis linux-tkg di direktori user
USER_HOME="/mnt/home/$USERNAME"
mkdir -p "$USER_HOME/kernel-build"

# Salin customization.cfg yang sudah dikonfigurasikan
cat <<EOF > "$USER_HOME/kernel-build/customization.cfg"
_cpusched="bore"
_compiler="llvm"
_lto_mode="full"
_use_polly="true"
_march="x86-64-v3"
_cpu_mitigations="false"
USER_KERNEL_CONFIG="CONFIG_HZ_1000=y\nCONFIG_PREEMPT=y\nCONFIG_PREEMPT_BUILD=y"
_kernel_base="zen"
EOF

# Tulis skrip instruksi kompilasi kernel bagi pengguna
cat <<EOF > "$USER_HOME/kernel-build/compile_kernel.sh"
#!/bin/bash
echo "=== Kompilasi Kernel linux-tkg-zen ==="
echo "Mengunduh source code linux-tkg..."
git clone https://github.com/Frogging-Family/linux-tkg.git /tmp/linux-tkg
cp customization.cfg /tmp/linux-tkg/
cd /tmp/linux-tkg
echo "Memulai kompilasi (Bisa memakan waktu beberapa jam)..."
makepkg -si --noconfirm
EOF
chmod +x "$USER_HOME/kernel-build/compile_kernel.sh"

# Install package sched_ext untuk LAVD Scheduler
arch-chroot /mnt pacman -S --noconfirm sched-ext-utility || echo "sched-ext-utility tidak tersedia di repo default, bisa dipasang manual lewat AUR"

# Salin unit systemd scx-lavd.service
cat <<EOF > /mnt/etc/systemd/system/scx-lavd.service
[Unit]
Description=LAVD Scheduler (Latency-criticality Aware Virtual Deadline via sched_ext)
After=multi-user.target

[Service]
Type=simple
ExecStart=/usr/bin/scx_lavd
Restart=always
RestartSec=2
CPUSchedulingPolicy=rr
CPUSchedulingPriority=99

[Install]
WantedBy=multi-user.target
EOF
arch-chroot /mnt systemctl enable scx-lavd.service || echo "Gagal mengaktifkan scx-lavd service (akan berfungsi penuh setelah scx scheduler terinstal)"

# Kepemilikan folder build kernel oleh user target
chroot /mnt chown -R 1000:1000 "/home/$USERNAME/kernel-build"

# 11. Bootloader Installation (Grub)
report_progress 92 "Mengonfigurasi Bootloader GRUB..."
arch-chroot /mnt grub-install --target=x86_64-efi --efi-directory=/boot --bootloader-id=GRUB
arch-chroot /mnt grub-mkconfig -o /boot/grub/grub.cfg

# 12. Mengaktifkan Layanan Sistem yang Diperlukan
report_progress 96 "Mengaktifkan layanan networkmanager..."
arch-chroot /mnt systemctl enable NetworkManager
arch-chroot /mnt systemctl enable iwd

report_progress 100 "Instalasi selesai!"
echo "Sistem Arch Gaming Linux kustom berhasil dipasang!"
echo "Silakan reboot dan cabut USB flashdisk Anda."
