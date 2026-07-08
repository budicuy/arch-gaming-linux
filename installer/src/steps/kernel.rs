use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::{AppState, KernelOption};

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
    let header = Paragraph::new("PILIHAN KERNEL DIOPTIMALKAN")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Main layout
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    // Left: Kernel selection (default is linux-tkg-zen)
    let options = vec![KernelOption::LinuxTkgZen];
    let items: Vec<ListItem> = options.iter().map(|opt| {
        let prefix = if state.kernel == *opt { "● " } else { "○ " };
        ListItem::new(format!("{}{}", prefix, opt))
    }).collect();

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let kernel_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Pilihan Kernel Gaming "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(kernel_list, main_layout[0], &mut list_state);

    // Right: Kernel Details
    let details = vec![
        "INFORMASI KERNEL linux-tkg-zen:".to_string(),
        "".to_string(),
        "Kernel ini di-compile khusus dengan fitur-fitur gaming ekstrim:".to_string(),
        "  • BORE CPU Scheduler (Burst-Oriented Response Enhancer)".to_string(),
        "    Menjaga tingkat frame-rate (FPS) tetap konsisten saat CPU mengalami beban multitasking tinggi.".to_string(),
        "  • Clang Compiler & LTO Full + Polly Loop Optimization".to_string(),
        "    Optimalisasi instruksi mesin tingkat tinggi untuk performa CPU termaksimalkan.".to_string(),
        "  • Optimasi Arsitektur x86-64-v3 (AVX2)".to_string(),
        "    Memanfaatkan set instruksi modern untuk pengolahan data game.".to_string(),
        "  • Frekuensi Timer 1000Hz".to_string(),
        "    Meningkatkan responsivitas interupsi kernel secara real-time.".to_string(),
        "  • Menonaktifkan CPU Mitigations".to_string(),
        "    Membebaskan performa core CPU dari batasan keamanan bawaan kernel standar (peningkatan performa 5-15%).".to_string(),
    ].join("\n");

    let detail_panel = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Konfigurasi & Optimasi "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(detail_panel, main_layout[1]);

    // Footer
    let footer_text = "ENTER: Lanjut | ESC: Kembali";
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
