use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::{AppState, DesktopEnv};

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
    let header = Paragraph::new("PILIHAN DESKTOP / WINDOW MANAGER")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Main split
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(chunks[1]);

    // Left: Options
    let options = vec![
        DesktopEnv::Hyprland,
        DesktopEnv::Sway,
        DesktopEnv::KdePlasma,
        DesktopEnv::Gnome,
        DesktopEnv::Xfce,
        DesktopEnv::GamescopeOnly,
    ];

    let items: Vec<ListItem> = options.iter().map(|opt| {
        let prefix = if state.desktop == *opt { "● " } else { "○ " };
        ListItem::new(format!("{}{}", prefix, opt))
    }).collect();

    let mut list_state = ListState::default();
    let idx = options.iter().position(|&o| o == state.desktop).unwrap_or(0);
    list_state.select(Some(idx));

    let desktop_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Pilihan Lingkungan Desktop "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(desktop_list, main_layout[0], &mut list_state);

    // Right: Detail/Description Panel
    let details = match state.desktop {
        DesktopEnv::Hyprland => vec![
            "HYPRLAND (Wayland)".to_string(),
            "".to_string(),
            "Hyprland adalah compositor Wayland dinamis berbasis wlroots yang modern, fluid, dan sangat dapat dikustomisasi.".to_string(),
            "Memiliki performa grafis rendering yang mulus dengan animasi yang indah, optimal untuk monitor refresh-rate tinggi.".to_string(),
            "Direkomendasikan untuk GPU modern (AMD/NVIDIA).".to_string(),
        ],
        DesktopEnv::Sway => vec![
            "SWAY (Wayland)".to_string(),
            "".to_string(),
            "Sway adalah tiling Wayland compositor yang kompatibel langsung dengan i3 window manager.".to_string(),
            "Sangat ringan, stabil, dan efisien dalam penggunaan resource memori serta CPU.".to_string(),
            "Cocok untuk minimalis yang menginginkan performa maksimal tanpa bloatware.".to_string(),
        ],
        DesktopEnv::KdePlasma => vec![
            "KDE PLASMA (Wayland/X11)".to_string(),
            "".to_string(),
            "KDE Plasma adalah desktop environment yang sangat kaya fitur, modern, dan sangat mudah dikustomisasi secara visual.".to_string(),
            "Kini menggunakan Wayland secara default pada versi terbarunya, memberikan performa desktop yang mulus.".to_string(),
            "Cocok untuk pengguna yang menginginkan tampilan desktop tradisional ala Windows namun tetap modern dan fleksibel.".to_string(),
        ],
        DesktopEnv::Gnome => vec![
            "GNOME (Wayland/X11)".to_string(),
            "".to_string(),
            "GNOME adalah desktop environment modern yang sangat terpoles (polished) dengan alur kerja berbasis gestur yang unik.".to_string(),
            "Sangat stabil, memiliki integrasi ekosistem aplikasi yang solid, dan dukungan Wayland yang matang.".to_string(),
            "Cocok untuk pengguna yang menyukai kesederhanaan, keindahan visual, dan UI yang bersih.".to_string(),
        ],
        DesktopEnv::Xfce => vec![
            "XFCE (Classic X11)".to_string(),
            "".to_string(),
            "XFCE adalah desktop environment klasik yang sangat ringan, stabil, dan mudah digunakan.".to_string(),
            "Menggunakan protokol X11 tradisional yang memiliki kompatibilitas sangat tinggi dengan game-game jadul.".to_string(),
        ],
        DesktopEnv::GamescopeOnly => vec![
            "GAMESCOPE ONLY (Micro-Compositor)".to_string(),
            "".to_string(),
            "Sistem akan langsung melakukan boot ke sesi micro-compositor Gamescope buatan Valve (seperti SteamOS pada Steam Deck).".to_string(),
            "Tidak ada panel desktop desktop tradisional, window manager, ataupun browser web.".to_string(),
            "Sangat optimal untuk PC gaming murni / konsol gaming ruang tamu karena seluruh resource dialokasikan hanya untuk Steam Big Picture.".to_string(),
        ],
    }.join("\n");

    let detail_panel = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Penjelasan Fitur "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(detail_panel, main_layout[1]);

    // Footer
    let footer_text = "ENTER: Lanjut | ESC: Kembali | Panah Up/Down: Pilih";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    let options = vec![
        DesktopEnv::Hyprland,
        DesktopEnv::Sway,
        DesktopEnv::KdePlasma,
        DesktopEnv::Gnome,
        DesktopEnv::Xfce,
        DesktopEnv::GamescopeOnly,
    ];
    let current_idx = options.iter().position(|&o| o == state.desktop).unwrap_or(0);

    match key.code {
        KeyCode::Up => {
            let next_idx = if current_idx > 0 { current_idx - 1 } else { options.len() - 1 };
            state.desktop = options[next_idx];
        }
        KeyCode::Down => {
            let next_idx = if current_idx < options.len() - 1 { current_idx + 1 } else { 0 };
            state.desktop = options[next_idx];
        }
        KeyCode::Enter => {
            state.current_step = state.current_step.next();
        }
        KeyCode::Esc | KeyCode::Backspace => {
            state.current_step = state.current_step.prev();
        }
        _ => {}
    }
}
