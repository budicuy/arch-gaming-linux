use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
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
    let header = Paragraph::new("KONFIGURASI PENGGUNA & HOS")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Input fields layout
    let form_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Hostname
            Constraint::Length(3), // Username
            Constraint::Length(3), // Password
            Constraint::Length(3), // Confirm Password
            Constraint::Min(0),    // Error message panel
        ])
        .split(chunks[1]);

    // Form inputs rendering
    let fields = vec![
        (" Hostname (Nama Komputer) ", &state.hostname, false),
        (" Username (Nama Pengguna) ", &state.username, false),
        (" Kata Sandi (Password) ", &state.password, true),
        (" Konfirmasi Kata Sandi ", &state.confirm_password, true),
    ];

    for (idx, (label, val, is_password)) in fields.iter().enumerate() {
        let display_val = if *is_password {
            format!("{}{}", "*".repeat(val.len()), if idx == state.active_input_field { "_" } else { "" })
        } else {
            format!("{}{}", val, if idx == state.active_input_field { "_" } else { "" })
        };

        let is_active = idx == state.active_input_field;
        let border_style = if is_active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_widget = Paragraph::new(display_val)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style)
                    .title(*label),
            );

        frame.render_widget(input_widget, form_layout[idx]);
    }

    // Render error if exists
    if let Some(ref err) = state.error_message {
        let err_widget = Paragraph::new(format!("Error: {}", err))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(err_widget, form_layout[4]);
    }

    // Footer
    let footer_text = "TAB / ENTER: Pindah Field | ESC: Kembali";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            state.active_input_field = (state.active_input_field + 1) % 4;
            state.error_message = None;
        }
        KeyCode::Up => {
            if state.active_input_field > 0 {
                state.active_input_field -= 1;
            } else {
                state.active_input_field = 3;
            }
            state.error_message = None;
        }
        KeyCode::Down => {
            state.active_input_field = (state.active_input_field + 1) % 4;
            state.error_message = None;
        }
        KeyCode::Backspace => {
            match state.active_input_field {
                0 => { state.hostname.pop(); }
                1 => { state.username.pop(); }
                2 => { state.password.pop(); }
                3 => { state.confirm_password.pop(); }
                _ => {}
            }
        }
        KeyCode::Char(c) => {
            match state.active_input_field {
                0 => { if state.hostname.len() < 32 { state.hostname.push(c); } }
                1 => { if state.username.len() < 32 { state.username.push(c); } }
                2 => { if state.password.len() < 64 { state.password.push(c); } }
                3 => { if state.confirm_password.len() < 64 { state.confirm_password.push(c); } }
                _ => {}
            }
        }
        KeyCode::Enter => {
            if state.active_input_field < 3 {
                state.active_input_field += 1;
            } else {
                // Selesai input, validasi data
                if state.hostname.trim().is_empty() {
                    state.error_message = Some("Hostname tidak boleh kosong!".to_string());
                } else if state.username.trim().is_empty() {
                    state.error_message = Some("Username tidak boleh kosong!".to_string());
                } else if state.password.is_empty() {
                    state.error_message = Some("Password tidak boleh kosong!".to_string());
                } else if state.password != state.confirm_password {
                    state.error_message = Some("Konfirmasi Password tidak cocok!".to_string());
                } else {
                    state.error_message = None;
                    state.active_input_field = 0; // reset
                    state.current_step = state.current_step.next();
                }
            }
        }
        KeyCode::Esc => {
            state.current_step = state.current_step.prev();
            state.active_input_field = 0; // reset
            state.error_message = None;
        }
        _ => {}
    }
}
