use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::AppState;

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
    let header = Paragraph::new("RINGKASAN KONFIGURASI INSTALASI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Construct the rows for the Table
    let gaming_pkgs = {
        let mut pkgs = Vec::new();
        if state.install_steam { pkgs.push("Steam"); }
        if state.install_mangohud { pkgs.push("MangoHud"); }
        if state.install_gamemode { pkgs.push("Gamemode"); }
        if state.install_protonup { pkgs.push("ProtonUp"); }
        if state.install_wine { pkgs.push("Wine"); }
        if pkgs.is_empty() { "Tidak ada".to_string() } else { pkgs.join(", ") }
    };

    let wifi_status = if state.network_ssid.is_empty() {
        "Wired / Ethernet (Kabel)".to_string()
    } else {
        format!("Wi-Fi (SSID: {})", state.network_ssid)
    };

    let rows = vec![
        Row::new(vec![Cell::from("Disk Target").style(Style::default().fg(Color::Cyan)), Cell::from(format!("/dev/{}", state.selected_disk))]),
        Row::new(vec![Cell::from("Metode Partisi").style(Style::default().fg(Color::Cyan)), Cell::from(state.partition_type.to_string())]),
        Row::new(vec![Cell::from("Desktop / WM").style(Style::default().fg(Color::Cyan)), Cell::from(state.desktop.to_string())]),
        Row::new(vec![Cell::from("Kernel Utama").style(Style::default().fg(Color::Cyan)), Cell::from(state.kernel.to_string())]),
        Row::new(vec![Cell::from("Shell Sistem").style(Style::default().fg(Color::Cyan)), Cell::from(state.shell.to_string())]),
        Row::new(vec![Cell::from("Terminal Emulator").style(Style::default().fg(Color::Cyan)), Cell::from(state.terminal.to_string())]),
        Row::new(vec![Cell::from("Paket Gaming").style(Style::default().fg(Color::Cyan)), Cell::from(gaming_pkgs)]),
        Row::new(vec![Cell::from("Hostname").style(Style::default().fg(Color::Cyan)), Cell::from(state.hostname.clone())]),
        Row::new(vec![Cell::from("Username").style(Style::default().fg(Color::Cyan)), Cell::from(state.username.clone())]),
        Row::new(vec![Cell::from("Koneksi Jaringan").style(Style::default().fg(Color::Cyan)), Cell::from(wifi_status)]),
    ];

    let widths = [Constraint::Length(25), Constraint::Min(40)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Ringkasan Data yang Akan Ditulis ke Disk "),
        )
        .header(
            Row::new(vec![Cell::from("Parameter"), Cell::from("Nilai Konfigurasi")])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        )
        .column_spacing(2);

    // Let's divide chunks[1] to render warning under the table
    let main_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(chunks[1]);

    frame.render_widget(table, main_split[0]);

    let warning_text = "PERINGATAN: Menyetujui instalasi ini akan melakukan format ulang pada disk terpilih. Semua data pada disk tersebut akan terhapus secara permanen.";
    let warning_widget = Paragraph::new(warning_text)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(Color::Red)));
    frame.render_widget(warning_widget, main_split[1]);

    // Footer
    let footer_text = "ENTER: Setujui & Mulai Instalasi | ESC: Kembali";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            state.current_step = state.current_step.next();
        }
        KeyCode::Esc | KeyCode::Backspace => {
            state.current_step = state.current_step.prev();
        }
        _ => {}
    }
}
