mod tui;
mod state;
mod steps;

use state::{AppState, Step};
use std::io;
use tokio::sync::mpsc::{self, UnboundedSender};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::process::Command as TokioCommand;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug)]
enum AppEvent {
    Key(KeyEvent),
    WifiScanDone(Vec<String>),
    InstallProgress(f64),
    InstallStatus(String),
    InstallLog(String),
    InstallDone(Result<(), String>),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Daftarkan panic hook agar terminal kembali ke kondisi normal jika terjadi crash
    std::panic::set_hook(Box::new(|panic_info| {
        let _ = tui::restore();
        eprintln!("==================================================");
        eprintln!(" KESALAHAN VITAL: Installer mengalami panic!");
        eprintln!("==================================================");
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Detail: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Detail: {}", s);
        } else {
            eprintln!("Detail: Tidak diketahui");
        }
        if let Some(location) = panic_info.location() {
            eprintln!("Lokasi: {}:{}:{}", location.file(), location.line(), location.column());
        }
        eprintln!("==================================================");
    }));

    // Inisialisasi TUI terminal
    let mut terminal = tui::init()?;
    
    // Inisialisasi State
    let mut state = AppState::new();
    
    // Scan awal disk fisik
    steps::partition::trigger_disk_scan(&mut state);
    
    // Scan awal wifi di latar belakang
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();
    
    // Spawn thread untuk menangani keyboard input dari Crossterm
    let key_tx = event_tx.clone();
    std::thread::spawn(move || {
        loop {
            if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                // Filter hanya key press event
                if key.kind == crossterm::event::KeyEventKind::Press {
                    let _ = key_tx.send(AppEvent::Key(key));
                }
            }
        }
    });

    // Jalankan scan WiFi awal secara asinkron
    let wifi_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut mock_list = vec![
            "ROG_Gaming_5G".to_string(),
            "Home_Fiber_Ultra".to_string(),
            "Cybercafe_Pro_Gamer".to_string(),
            "Arch_Linux_Hotspot".to_string(),
        ];
        
        #[cfg(target_os = "linux")]
        {
            let _ = tokio::process::Command::new("iwctl")
                .args(["station", "wlan0", "scan"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
            if let Ok(output) = tokio::process::Command::new("iwctl").args(["station", "wlan0", "get-networks"]).output().await {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                let mut list = Vec::new();
                for line in stdout_str.lines() {
                    let clean_line = strip_ansi(line);
                    let trimmed = clean_line.trim();
                    if trimmed.is_empty() 
                        || trimmed.starts_with("---") 
                        || trimmed.starts_with("Name") 
                        || trimmed.contains("Could not") 
                        || trimmed.contains("Failed") 
                        || trimmed.contains("error") 
                        || trimmed.contains("Error") 
                    {
                        continue;
                    }
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if !parts.is_empty() {
                        let ssid = parts[0].to_string();
                        if !list.contains(&ssid) && ssid != ">" && !ssid.is_empty() {
                            list.push(ssid);
                        }
                    }
                }
                if !list.is_empty() {
                    mock_list = list;
                }
            }
        }
        
        let _ = wifi_tx.send(AppEvent::WifiScanDone(mock_list));
    });

    // Loop utama rendering dan event handling
    loop {
        // Draw UI
        terminal.draw(|frame| {
            let area = frame.area();
            // Bersihkan terminal untuk mencegah ghost text dari langkah sebelumnya
            frame.render_widget(ratatui::widgets::Clear, area);
            match state.current_step {
                Step::Welcome => steps::welcome::draw(frame, &state, area),
                Step::Network => steps::network::draw(frame, &state, area),
                Step::Partition => steps::partition::draw(frame, &state, area),
                Step::Desktop => steps::desktop::draw(frame, &state, area),
                Step::Kernel => steps::kernel::draw(frame, &state, area),
                Step::Shell => steps::shell::draw(frame, &state, area),
                Step::Gaming => steps::gaming::draw(frame, &state, area),
                Step::User => steps::user::draw(frame, &state, area),
                Step::Summary => steps::summary::draw(frame, &state, area),
                Step::Install => steps::install::draw(frame, &state, area),
            }
        })?;

        // Mulai proses instalasi jika masuk langkah Install dan belum berjalan
        if state.current_step == Step::Install && !state.is_installing && !state.install_completed {
            state.is_installing = true;
            spawn_installation_task(&state, event_tx.clone());
        }

        // Tangani event yang masuk
        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Key(key) => {
                    // Global shortcut: Ctrl + C untuk keluar dari installer
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        break;
                    }
                    
                    // Teruskan event ke langkah yang aktif
                    match state.current_step {
                        Step::Welcome => steps::welcome::handle_key(&mut state, key),
                        Step::Network => steps::network::handle_key(&mut state, key),
                        Step::Partition => steps::partition::handle_key(&mut state, key),
                        Step::Desktop => steps::desktop::handle_key(&mut state, key),
                        Step::Kernel => steps::kernel::handle_key(&mut state, key),
                        Step::Shell => steps::shell::handle_key(&mut state, key),
                        Step::Gaming => steps::gaming::handle_key(&mut state, key),
                        Step::User => steps::user::handle_key(&mut state, key),
                        Step::Summary => steps::summary::handle_key(&mut state, key),
                        Step::Install => steps::install::handle_key(&mut state, key),
                    }
                }
                AppEvent::WifiScanDone(ssids) => {
                    state.ssids = ssids;
                    state.scanning_wifi = false;
                }
                AppEvent::InstallProgress(prog) => {
                    state.install_progress = prog;
                }
                AppEvent::InstallStatus(status) => {
                    state.install_status = status;
                }
                AppEvent::InstallLog(log_line) => {
                    state.install_logs.push(log_line);
                }
                AppEvent::InstallDone(result) => {
                    state.is_installing = false;
                    match result {
                        Ok(_) => {
                            state.install_completed = true;
                            state.install_progress = 1.0;
                            state.install_status = "Instalasi selesai dengan sukses! Sistem siap di-reboot.".to_string();
                        }
                        Err(err) => {
                            state.error_message = Some(err);
                        }
                    }
                }
            }
        }
    }

    // Kembalikan kondisi terminal asal
    tui::restore()?;
    Ok(())
}

fn spawn_installation_task(state: &AppState, tx: UnboundedSender<AppEvent>) {
    // Ambil semua variabel yang dibutuhkan
    let disk = state.selected_disk.clone();
    let part_method = format!("{:?}", state.partition_type);
    let desktop = format!("{:?}", state.desktop);
    let kernel = format!("{:?}", state.kernel);
    let shell = format!("{:?}", state.shell);
    let terminal_app = format!("{:?}", state.terminal);
    let hostname = state.hostname.clone();
    let username = state.username.clone();
    let password = state.password.clone();
    
    let install_steam = state.install_steam;
    let install_mangohud = state.install_mangohud;
    let install_gamemode = state.install_gamemode;
    let install_protonup = state.install_protonup;
    let install_wine = state.install_wine;

    tokio::spawn(async move {
        // Cari skrip install_base.sh
        // Di lingkungan Archiso, ia akan diletakkan di /usr/local/bin/install_base.sh
        // Untuk simulasi lokal, jika skrip tidak ada, kita jalankan dummy installer
        let script_path = "/usr/local/bin/install_base.sh";
        
        let script_exists = std::path::Path::new(script_path).exists();
        
        if !script_exists {
            // Jalankan mode simulasi dummy untuk testing lokal
            run_dummy_installer(tx).await;
            return;
        }

        let mut cmd = TokioCommand::new("bash");
        cmd.arg(script_path)
           .env("DISK", disk)
           .env("PART_METHOD", part_method)
           .env("DESKTOP", desktop)
           .env("KERNEL", kernel)
           .env("SHELL", shell)
           .env("TERMINAL", terminal_app)
           .env("HOSTNAME", hostname)
           .env("USERNAME", username)
           .env("PASSWORD", password)
           .env("STEAM", install_steam.to_string())
           .env("MANGOHUD", install_mangohud.to_string())
           .env("GAMEMODE", install_gamemode.to_string())
           .env("PROTONUP", install_protonup.to_string())
           .env("WINE", install_wine.to_string())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(AppEvent::InstallDone(Err(format!("Gagal mengeksekusi skrip instalasi: {}", e))));
                return;
            }
        };

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        let tx_log = tx.clone();
        let tx_prog = tx.clone();
        
        // Task untuk membaca stdout
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.starts_with("[STATUS] ") {
                    let status = line["[STATUS] ".len()..].to_string();
                    let _ = tx_prog.send(AppEvent::InstallStatus(status));
                } else if line.starts_with("[PROGRESS] ") {
                    if let Ok(prog_val) = line["[PROGRESS] ".len()..].trim().parse::<f64>() {
                        let _ = tx_prog.send(AppEvent::InstallProgress(prog_val / 100.0));
                    }
                } else {
                    let _ = tx_log.send(AppEvent::InstallLog(line));
                }
            }
        });

        // Task untuk membaca stderr
        let tx_err_log = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_err_log.send(AppEvent::InstallLog(format!("[ERROR] {}", line)));
            }
        });

        // Tunggu hingga proses selesai
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    let _ = tx.send(AppEvent::InstallDone(Ok(())));
                } else {
                    let _ = tx.send(AppEvent::InstallDone(Err(format!("Skrip instalasi keluar dengan kode error: {:?}", status.code()))));
                }
            }
            Err(e) => {
                let _ = tx.send(AppEvent::InstallDone(Err(format!("Gagal menunggu proses instalasi: {}", e))));
            }
        }
    });
}

// Simulasi instalasi jika dijalankan secara lokal (non-Archiso)
async fn run_dummy_installer(tx: UnboundedSender<AppEvent>) {
    let dummy_steps = vec![
        ("Mempersiapkan drive target...", 5.0),
        ("Membuat tabel partisi GPT baru...", 10.0),
        ("Membuat partisi EFI System & BTRFS root...", 15.0),
        ("Memformat partisi EFI ke FAT32...", 20.0),
        ("Memformat partisi root ke BTRFS...", 25.0),
        ("Membuat subvolume BTRFS (@, @home, @cache, @log)...", 30.0),
        ("Melakukan mount subvolume...", 35.0),
        ("Memasang base system & paket esensial (pacstrap)...", 45.0),
        ("Memasang kernel kustom linux-tkg-zen...", 60.0),
        ("Membuat berkas fstab...", 65.0),
        ("Mengonfigurasi locale, timezone, dan console...", 70.0),
        ("Membuat user gaming baru & menyetel hak sudo...", 75.0),
        ("Mengonfigurasi bootloader Grub...", 80.0),
        ("Mengunduh & mengonfigurasi Desktop/WM terpilih...", 85.0),
        ("Mengaktifkan tweaks gaming sysctl (max_map_count, swappiness)...", 90.0),
        ("Memasang driver grafis & paket gaming (Steam, MangoHud)...", 95.0),
        ("Menyelesaikan setup...", 100.0),
    ];

    for (status, progress) in dummy_steps {
        let _ = tx.send(AppEvent::InstallStatus(status.to_string()));
        let _ = tx.send(AppEvent::InstallProgress(progress / 100.0));
        
        // Log baris simulasi
        let _ = tx.send(AppEvent::InstallLog(format!("Executing: {}", status)));
        if progress == 45.0 {
            // Simulasi output pacstrap
            let _ = tx.send(AppEvent::InstallLog(" -> Installing base (1/5)".to_string()));
            let _ = tx.send(AppEvent::InstallLog(" -> Installing linux-firmware (2/5)".to_string()));
            let _ = tx.send(AppEvent::InstallLog(" -> Installing pipewire (3/5)".to_string()));
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        } else if progress == 60.0 {
            // Simulasi compiler kernel
            let _ = tx.send(AppEvent::InstallLog(" -> Injecting linux-tkg-zen-v3 configuration...".to_string()));
            let _ = tx.send(AppEvent::InstallLog(" -> Compiling kernel with Clang + LTO + Polly...".to_string()));
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    let _ = tx.send(AppEvent::InstallDone(Ok(())));
}

fn strip_ansi(input: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_welcome_step() {
        let mut state = AppState::new();
        assert_eq!(state.current_step, Step::Welcome);
        
        // Simulasikan menekan Enter pada Welcome screen
        steps::welcome::handle_key(&mut state, make_key_event(KeyCode::Enter));
        assert_eq!(state.current_step, Step::Network);
    }

    #[test]
    fn test_network_step_empty_ssids() {
        let mut state = AppState::new();
        state.current_step = Step::Network;
        state.ssids.clear();
        
        // Jika list SSID kosong, Enter harus melewati Wi-Fi dan lanjut ke Partition
        steps::network::handle_key(&mut state, make_key_event(KeyCode::Enter));
        assert_eq!(state.current_step, Step::Partition);
    }

    #[test]
    fn test_desktop_selection() {
        let mut state = AppState::new();
        state.current_step = Step::Desktop;
        state.desktop = state::DesktopEnv::Hyprland;
        
        // Panah Down harus mengganti desktop dari Hyprland ke Sway
        steps::desktop::handle_key(&mut state, make_key_event(KeyCode::Down));
        assert_eq!(state.desktop, state::DesktopEnv::Sway);
        
        // Panah Up harus mengganti desktop kembali ke Hyprland
        steps::desktop::handle_key(&mut state, make_key_event(KeyCode::Up));
        assert_eq!(state.desktop, state::DesktopEnv::Hyprland);
    }
    
    #[test]
    fn test_user_validation_success() {
        let mut state = AppState::new();
        state.current_step = Step::User;
        state.hostname = "test-host".to_string();
        state.username = "test-user".to_string();
        state.password = "pass123".to_string();
        state.confirm_password = "pass123".to_string();
        state.active_input_field = 3; // Pada input terakhir (Confirm Password)
        
        steps::user::handle_key(&mut state, make_key_event(KeyCode::Enter));
        assert_eq!(state.current_step, Step::Summary);
        assert!(state.error_message.is_none());
    }

    #[test]
    fn test_user_validation_password_mismatch() {
        let mut state = AppState::new();
        state.current_step = Step::User;
        state.hostname = "test-host".to_string();
        state.username = "test-user".to_string();
        state.password = "pass123".to_string();
        state.confirm_password = "different".to_string();
        state.active_input_field = 3;
        
        steps::user::handle_key(&mut state, make_key_event(KeyCode::Enter));
        assert_eq!(state.current_step, Step::User); // Tidak boleh lanjut
        assert!(state.error_message.is_some());
    }
}
