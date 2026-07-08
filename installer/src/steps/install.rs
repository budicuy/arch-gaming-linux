use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::AppState;

pub fn draw(frame: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3), // Progress Bar
            Constraint::Length(1), // Status Message
            Constraint::Min(0),    // Logs
            Constraint::Length(3), // Footer / Action button
        ])
        .split(area);

    // Header
    let header = Paragraph::new("PROSES INSTALASI SISTEM")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Progress Bar (Gauge)
    let percent = (state.install_progress * 100.0).clamp(0.0, 100.0) as u16;
    let gauge_color = if state.install_completed {
        Color::Green
    } else if state.error_message.is_some() {
        Color::Red
    } else {
        Color::Cyan
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Progress "))
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .percent(percent);
    frame.render_widget(gauge, chunks[1]);

    // Status Message
    let status_style = if state.error_message.is_some() {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if state.install_completed {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Yellow)
    };

    let status_text = if let Some(ref err) = state.error_message {
        format!("ERROR: {}", err)
    } else {
        format!("Status: {}", state.install_status)
    };

    let status_p = Paragraph::new(status_text)
        .style(status_style)
        .alignment(Alignment::Left);
    frame.render_widget(status_p, chunks[2]);

    // Logs Panel
    let log_items: Vec<ListItem> = state.install_logs.iter().rev().take(30).map(|log| {
        ListItem::new(log.as_str())
    }).collect();

    let logs_list = List::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Output Log Instalasi "),
        );
    frame.render_widget(logs_list, chunks[3]);

    // Footer
    let footer_text = if state.install_completed {
        "Instalasi Selesai! Tekan [ENTER] untuk reboot sistem."
    } else if state.error_message.is_some() {
        "Instalasi Gagal. Tekan [ESC] untuk kembali ke ringkasan."
    } else {
        "Menginstal... Harap jangan mematikan komputer atau mencabut USB ISO."
    };

    let footer_bg = if state.install_completed {
        Color::Green
    } else if state.error_message.is_some() {
        Color::Red
    } else {
        Color::Blue
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::White).bg(footer_bg).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    if state.install_completed && key.code == KeyCode::Enter {
        // Lakukan reboot sistem
        let _ = std::process::Command::new("reboot").status();
    } else if state.error_message.is_some() && (key.code == KeyCode::Esc || key.code == KeyCode::Backspace) {
        state.error_message = None;
        state.is_installing = false;
        state.install_progress = 0.0;
        state.install_logs.clear();
        state.install_status = "Menunggu persetujuan...".to_string();
        state.current_step = state.current_step.prev();
    }
}
