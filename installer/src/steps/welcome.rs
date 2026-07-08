use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use crate::state::AppState;

pub fn draw(frame: &mut Frame, _state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    let header = Paragraph::new("ARCH GAMING LINUX CUSTOM INSTALLER")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // ASCII Art
    let ascii_art = r#"
    _    ____   ____ _   _    ____    _    __  __ ___ _   _  ____ 
   / \  |  _ \ / ___| | | |  / ___|  / \  |  \/  |_ _| \ | |/ ___|
  / _ \ | |_) | |   | |_| | | |  _  / _ \ | |\/| || ||  \| | |  _ 
 / ___ \|  _ <| |___|  _  | | |_| |/ ___ \| |  | || || |\  | |_| |
/_/   \_\_| \_\\____|_| |_|  \____/_/   \_\_|  |_|___|_| \_|\____|
    "#;
    
    let ascii_paragraph = Paragraph::new(ascii_art)
        .style(Style::default().fg(Color::LightMagenta))
        .alignment(Alignment::Center);
    frame.render_widget(ascii_paragraph, chunks[1]);

    // Subtitle
    let subtitle = Paragraph::new("BASE ISO KUSTOM ULTRA-MINIMALIS & DIOPTIMALKAN UNTUK ULTRA-LOW LATENCY")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM))
        .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[2]);

    // Body / Description
    let welcome_text = vec![
        "".to_string(),
        "Selamat datang di setup wizard Arch Gaming Linux kustom Anda!".to_string(),
        "Sistem ini dirancang khusus untuk meminimalkan input lag, meningkatkan FPS,".to_string(),
        "dan memberikan performa kernel real-time yang stabil untuk sesi gaming intensif.".to_string(),
        "".to_string(),
        "Fitur Utama yang akan dikonfigurasi:".to_string(),
        "  • Kernel linux-tkg-zen dengan BORE Scheduler & optimasi arsitektur x86-64-v3".to_string(),
        "  • Clang LTO + Polly Compiler Optimization & Timer Frequency 1000Hz".to_string(),
        "  • Lingkungan grafis minimalis (Hyprland, Sway, atau Gamescope-only untuk gaming murni)".to_string(),
        "  • Tuning parameter memori (vm.max_map_count) & network backlog optimal secara otomatis".to_string(),
        "".to_string(),
        "Navigasi Dasar:".to_string(),
        "  • [Enter] / [Tab] : Lanjut ke langkah berikutnya".to_string(),
        "  • [Esc] / [Backspace] : Kembali ke langkah sebelumnya".to_string(),
        "  • [Panah Atas/Bawah] : Navigasi daftar pilihan".to_string(),
        "  • [Space] : Pilih/Batalkan pilihan item checkbox".to_string(),
        "  • [Ctrl + C] : Keluar dari installer".to_string(),
    ].join("\n");

    let body = Paragraph::new(welcome_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Informasi Penting "),
        )
        .alignment(Alignment::Left)
        .scroll((0, 0));
    
    frame.render_widget(body, chunks[3]);

    // Footer
    let footer_text = "Tekan [ENTER] untuk Mulai Konfigurasi";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}

pub fn handle_key(state: &mut AppState, key: crossterm::event::KeyEvent) {
    if key.code == crossterm::event::KeyCode::Enter {
        state.current_step = state.current_step.next();
    }
}
