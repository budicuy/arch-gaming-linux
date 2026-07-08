use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
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
    let header = Paragraph::new("PAKET GAMING & COMPATIBILITY LAYER")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Main layout splits
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(chunks[1]);

    // Left Panel: Checkbox list
    let list_items = vec![
        (state.install_steam, "Steam (Platform Distribusi Game Digital)"),
        (state.install_mangohud, "MangoHud (HUD Pemantau FPS, Suhu, & Beban Hardware)"),
        (state.install_gamemode, "Feral GameMode (Tuning Kernel & CPU Governor Otomatis)"),
        (state.install_protonup, "ProtonUp-Qt (Manajer Instalasi GE-Proton & Wine-GE)"),
        (state.install_wine, "Wine + Winetricks (Layer Kompatibilitas Aplikasi/Game Windows)"),
    ];

    let items: Vec<ListItem> = list_items.iter().enumerate().map(|(idx, (checked, label))| {
        let box_char = if *checked { "[x] " } else { "[ ] " };
        let item_text = format!("{}{}", box_char, label);
        if idx == state.list_selected {
            ListItem::new(item_text).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        } else {
            ListItem::new(item_text)
        }
    }).collect();

    let mut list_state = ListState::default();
    list_state.select(Some(state.list_selected));

    let gaming_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Pilihan Paket Tambahan (Tekan [SPACE] untuk memilih) "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(gaming_list, main_layout[0], &mut list_state);

    // Right Panel: Details
    let details = match state.list_selected {
        0 => vec![
            "STEAM".to_string(),
            "".to_string(),
            "Platform utama untuk gaming di Linux.".to_string(),
            "Dilengkapi dengan Proton (WINE buatan Valve) untuk menjalankan ribuan game Windows secara langsung dengan performa mendekati native.".to_string(),
        ],
        1 => vec![
            "MANGOHUD".to_string(),
            "".to_string(),
            "Vulkan/OpenGL overlay untuk memonitor performa secara real-time saat bermain game.".to_string(),
            "Menampilkan statistik krusial seperti FPS, frame timing, suhu CPU/GPU, dan persentase penggunaan RAM/VRAM.".to_string(),
        ],
        2 => vec![
            "FERAL GAMEMODE".to_string(),
            "".to_string(),
            "Daemon sistem yang mengoptimalkan PC saat game dimulai.".to_string(),
            "Secara otomatis mengubah CPU governor ke 'performance', mengoptimalkan prioritas penjadwalan proses (renice), meningkatkan batas I/O disk, serta menonaktifkan screensaver.".to_string(),
        ],
        3 => vec![
            "PROTONUP-QT".to_string(),
            "".to_string(),
            "Aplikasi GUI/TUI yang memudahkan pengunduhan versi Proton kustom.".to_string(),
            "Memungkinkan instalasi GE-Proton (Proton buatan GloriousEggroll) yang memiliki patch kompatibilitas game terbaru yang belum dirilis di Proton resmi.".to_string(),
        ],
        4 => vec![
            "WINE & WINETRICKS".to_string(),
            "".to_string(),
            "Penerjemah API Windows ke sistem operasi POSIX (Linux).".to_string(),
            "Membantu menjalankan launcher pihak ketiga (Epic Games, GOG Galaxy, EA Desktop) atau game mandiri non-Steam.".to_string(),
        ],
        _ => vec![],
    }.join("\n");

    let detail_panel = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Informasi Paket "),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(detail_panel, main_layout[1]);

    // Footer
    let footer_text = "SPACE: Toggle Pilihan | ENTER: Lanjut | ESC: Kembali";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Up => {
            if state.list_selected > 0 {
                state.list_selected -= 1;
            } else {
                state.list_selected = 4; // max item (0..4)
            }
        }
        KeyCode::Down => {
            if state.list_selected < 4 {
                state.list_selected += 1;
            } else {
                state.list_selected = 0;
            }
        }
        KeyCode::Char(' ') => {
            // Toggle selection
            match state.list_selected {
                0 => state.install_steam = !state.install_steam,
                1 => state.install_mangohud = !state.install_mangohud,
                2 => state.install_gamemode = !state.install_gamemode,
                3 => state.install_protonup = !state.install_protonup,
                4 => state.install_wine = !state.install_wine,
                _ => {}
            }
        }
        KeyCode::Enter => {
            state.current_step = state.current_step.next();
            state.list_selected = 0; // reset
        }
        KeyCode::Esc | KeyCode::Backspace => {
            state.current_step = state.current_step.prev();
            state.list_selected = 0; // reset
        }
        _ => {}
    }
}
