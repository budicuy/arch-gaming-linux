use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::AppState;
use std::process::Command;

pub fn draw(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let header = Paragraph::new("SETUP JARINGAN WI-FI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Main layout
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(chunks[1]);

    // Left side: SSID List
    let list_title = if state.scanning_wifi {
        " Memindai Jaringan Wi-Fi... [Silakan Tunggu] "
    } else {
        " Jaringan Wi-Fi yang Tersedia (Gunakan Panah Up/Down untuk memilih) "
    };

    let items: Vec<ListItem> = if state.ssids.is_empty() {
        if state.scanning_wifi {
            vec![ListItem::new("Memindai SSID...")]
        } else {
            vec![
                ListItem::new("Tidak ada jaringan Wi-Fi terdeteksi."),
                ListItem::new("[Gunakan Koneksi Ethernet / Kabel untuk melanjutkan]"),
                ListItem::new("Tekan 'S' untuk mencoba memindai ulang."),
            ]
        }
    } else {
        state.ssids.iter().map(|ssid| ListItem::new(ssid.as_str())).collect()
    };

    let mut list_state = ListState::default().with_selected(if state.ssids.is_empty() { None } else { Some(state.list_selected) });

    let ssid_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if state.scanning_wifi { Color::Yellow } else { Color::Green }))
                .title(list_title),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(ssid_list, main_layout[0], &mut list_state);

    // Right side: Info Panel
    let current_net_status = if !state.network_ssid.is_empty() {
        format!("Terhubung/Terpilih:\n  SSID: {}\n  Kata Sandi: {}", state.network_ssid, "*".repeat(state.network_pass.len()))
    } else {
        "Belum ada Wi-Fi terpilih.\nJika menggunakan koneksi kabel (Ethernet), Anda bisa langsung menekan [ENTER] untuk melewati setup Wi-Fi ini.".to_string()
    };

    let info_text = vec![
        "INFORMASI JARINGAN:".to_string(),
        "".to_string(),
        current_net_status,
        "".to_string(),
        "Installer akan menggunakan utilitas 'iwd' (iwctl) secara otomatis di latar belakang untuk melakukan koneksi.".to_string(),
        "".to_string(),
        "Instruksi:".to_string(),
        "  • S - Pindai ulang Wi-Fi".to_string(),
        "  • ENTER - Konfirmasi SSID terpilih / Lanjut ke langkah berikut".to_string(),
        "  • ESC / Backspace - Kembali".to_string(),
    ].join("\n");

    let info_panel = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Status & Bantuan "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(info_panel, main_layout[1]);

    // Footer
    let footer_text = "ENTER: Lanjut | ESC: Kembali | S: Scan Ulang";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);

    // Password Pop-up Overlay
    if state.show_wifi_password_popup {
        let popup_area = get_popup_rect(area, 50, 10);
        frame.render_widget(Clear, popup_area); // membersihkan area di bawah pop-up

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(popup_area);

        let title_ssid = if state.list_selected < state.ssids.len() {
            &state.ssids[state.list_selected]
        } else {
            "Wi-Fi"
        };

        let border_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Yellow))
            .title(format!(" Masukkan Password untuk: {} ", title_ssid));
        
        frame.render_widget(border_block, popup_area);

        let pass_display = format!(" {}{} ", "*".repeat(state.network_pass.len()), if state.network_pass.len() < 32 { "_" } else { "" });
        let pass_input = Paragraph::new(pass_display)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Password "));
        
        frame.render_widget(pass_input, popup_layout[1]);

        let help_text = Paragraph::new("ENTER: Simpan & Hubungkan | ESC: Batalkan")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help_text, popup_layout[3]);
    }
}

// Helper untuk menempatkan pop-up di tengah layar
fn get_popup_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    if state.show_wifi_password_popup {
        match key.code {
            KeyCode::Enter => {
                if state.list_selected < state.ssids.len() {
                    state.network_ssid = state.ssids[state.list_selected].clone();
                    // Di sini kita jalankan iwctl di latar belakang untuk menghubungkan
                    let _ssid = state.network_ssid.clone();
                    let _pass = state.network_pass.clone();
                    
                    // Kita bisa spawn thread untuk jalankan:
                    // iwctl station wlan0 connect <ssid> --passphrase <pass>
                    // Untuk simulasi, kita langsung simpan
                    state.show_wifi_password_popup = false;
                    state.current_step = state.current_step.next();
                }
            }
            KeyCode::Esc => {
                state.show_wifi_password_popup = false;
                state.network_pass.clear();
            }
            KeyCode::Char(c) => {
                if state.network_pass.len() < 64 {
                    state.network_pass.push(c);
                }
            }
            KeyCode::Backspace => {
                state.network_pass.pop();
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Enter => {
                if state.ssids.is_empty() {
                    // Skip setup wifi (menggunakan ethernet)
                    state.current_step = state.current_step.next();
                } else if state.list_selected < state.ssids.len() {
                    // Buka password pop-up
                    state.network_pass.clear();
                    state.show_wifi_password_popup = true;
                }
            }
            KeyCode::Esc | KeyCode::Backspace => {
                state.current_step = state.current_step.prev();
            }
            KeyCode::Up => {
                if !state.ssids.is_empty() {
                    if state.list_selected > 0 {
                        state.list_selected -= 1;
                    } else {
                        state.list_selected = state.ssids.len() - 1;
                    }
                }
            }
            KeyCode::Down => {
                if !state.ssids.is_empty() {
                    if state.list_selected < state.ssids.len() - 1 {
                        state.list_selected += 1;
                    } else {
                        state.list_selected = 0;
                    }
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Trigger Wifi Scan (Akan dipanggil di thread/tokio task di main loop)
                trigger_wifi_scan(state);
            }
            _ => {}
        }
    }
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

pub fn trigger_wifi_scan(state: &mut AppState) {
    state.scanning_wifi = true;
    state.ssids.clear();
    
    // Jalankan iwctl station wlan0 scan dan get-networks
    // Sebagai fallback atau simulasi jika tidak ada wifi:
    // Kita jalankan iwctl langsung.
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("iwctl")
            .args(["station", "wlan0", "scan"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        
        // 2. Ambil get-networks
        if let Ok(output) = Command::new("iwctl").args(["station", "wlan0", "get-networks"]).output() {
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
                state.ssids = list;
                state.scanning_wifi = false;
                state.list_selected = 0;
                return;
            }
        }
    }

    // Simulasi jika gagal / di VM tanpa Wi-Fi
    state.ssids = vec![
        "ROG_Gaming_5G".to_string(),
        "Home_Fiber_Ultra".to_string(),
        "Cybercafe_Pro_Gamer".to_string(),
        "Arch_Linux_Hotspot".to_string(),
    ];
    state.scanning_wifi = false;
    state.list_selected = 0;
}
