# Autostart the Rust TUI installer on TTY1 live session
if [[ -z $DISPLAY ]] && [[ $(tty) = /dev/tty1 ]]; then
    clear
    echo "==========================================="
    echo "  Memulai Arch Gaming Linux Installer TUI  "
    echo "==========================================="
    sleep 1
    
    # Eksekusi biner installer Rust
    if [ -f /usr/local/bin/gaming-installer ]; then
        /usr/local/bin/gaming-installer
    else
        echo "Error: Biner installer tidak ditemukan di /usr/local/bin/gaming-installer"
        echo "Membuka shell bash alternatif..."
        /bin/bash
    fi
fi
