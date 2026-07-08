use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::{AppState, DiskInfo, LsblkOutput, PartitionType};
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
    let header = Paragraph::new("PARTISI DISK & FORMATTING")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Split main content horizontally
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    // Left side: Disk Selection
    let disk_items: Vec<ListItem> = if state.disks.is_empty() {
        vec![ListItem::new("Tidak ada disk terdeteksi. Tekan 'R' untuk memuat ulang.")]
    } else {
        state.disks.iter().map(|disk| {
            let model = disk.model.clone().unwrap_or_else(|| "Unknown Model".to_string());
            ListItem::new(format!("  {} ({}) - {}", disk.name, disk.size, model))
        }).collect()
    };

    let mut disk_list_state = ListState::default();
    if !state.disks.is_empty() {
        // Temukan index disk yang terpilih saat ini
        if let Some(idx) = state.disks.iter().position(|d| d.name == state.selected_disk) {
            disk_list_state.select(Some(idx));
        } else {
            disk_list_state.select(Some(0));
        }
    }

    let disk_list = List::new(disk_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if state.active_input_field == 0 { Color::Cyan } else { Color::DarkGray }))
                .title(" 1. Pilih Disk Target "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(disk_list, main_layout[0], &mut disk_list_state);

    // Right side: Partition Type Selection
    let partition_options = vec![
        PartitionType::AutoBtrfs,
        PartitionType::AutoExt4,
        PartitionType::ManualCfdisk,
    ];

    let opt_items: Vec<ListItem> = partition_options.iter().map(|opt| {
        let prefix = if state.partition_type == *opt { "[x] " } else { "[ ] " };
        ListItem::new(format!("{}{}", prefix, opt))
    }).collect();

    let mut opt_list_state = ListState::default();
    if state.active_input_field == 1 {
        let idx = partition_options.iter().position(|&o| o == state.partition_type).unwrap_or(0);
        opt_list_state.select(Some(idx));
    }

    let opt_list = List::new(opt_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if state.active_input_field == 1 { Color::Cyan } else { Color::DarkGray }))
                .title(" 2. Metode Partisi "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(opt_list, main_layout[1], &mut opt_list_state);

    // Footer Help text
    let help_text = format!(
        "Disk Terpilih: /dev/{} | Metode: {}\n[Tab]: Pindah Panel | [Panah Up/Down]: Navigasi | [Space/Enter]: Pilih | [R]: Scan Ulang Disks",
        if state.selected_disk.is_empty() { "Belum dipilih" } else { &state.selected_disk },
        state.partition_type
    );
    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            // Berpindah panel: 0 = Disk Selection, 1 = Partition Type Selection
            state.active_input_field = if state.active_input_field == 0 { 1 } else { 0 };
        }
        KeyCode::Up => {
            if state.active_input_field == 0 && !state.disks.is_empty() {
                if let Some(idx) = state.disks.iter().position(|d| d.name == state.selected_disk) {
                    if idx > 0 {
                        state.selected_disk = state.disks[idx - 1].name.clone();
                    } else {
                        state.selected_disk = state.disks[state.disks.len() - 1].name.clone();
                    }
                } else {
                    state.selected_disk = state.disks[0].name.clone();
                }
            } else if state.active_input_field == 1 {
                let options = vec![PartitionType::AutoBtrfs, PartitionType::AutoExt4, PartitionType::ManualCfdisk];
                let current_idx = options.iter().position(|&o| o == state.partition_type).unwrap_or(0);
                let next_idx = if current_idx > 0 { current_idx - 1 } else { options.len() - 1 };
                state.partition_type = options[next_idx];
            }
        }
        KeyCode::Down => {
            if state.active_input_field == 0 && !state.disks.is_empty() {
                if let Some(idx) = state.disks.iter().position(|d| d.name == state.selected_disk) {
                    if idx < state.disks.len() - 1 {
                        state.selected_disk = state.disks[idx + 1].name.clone();
                    } else {
                        state.selected_disk = state.disks[0].name.clone();
                    }
                } else {
                    state.selected_disk = state.disks[0].name.clone();
                }
            } else if state.active_input_field == 1 {
                let options = vec![PartitionType::AutoBtrfs, PartitionType::AutoExt4, PartitionType::ManualCfdisk];
                let current_idx = options.iter().position(|&o| o == state.partition_type).unwrap_or(0);
                let next_idx = if current_idx < options.len() - 1 { current_idx + 1 } else { 0 };
                state.partition_type = options[next_idx];
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if state.active_input_field == 0 {
                // Di panel disk selection, menekan Enter memilih disk tersebut dan pindah ke panel metode partisi
                state.active_input_field = 1;
            } else if state.active_input_field == 1 {
                // Di panel metode partisi, menekan Enter mengonfirmasi dan lanjut ke langkah berikutnya
                // KECUALI jika tipe partisinya adalah ManualCfdisk, kita perlu menjalankan cfdisk terlebih dahulu.
                if state.partition_type == PartitionType::ManualCfdisk {
                    if !state.selected_disk.is_empty() {
                        let disk_path = format!("/dev/{}", state.selected_disk);
                        // Jalankan cfdisk
                        let _ = run_cfdisk(&disk_path);
                    }
                }
                state.current_step = state.current_step.next();
                state.active_input_field = 0; // reset
            }
        }
        KeyCode::Esc | KeyCode::Backspace => {
            state.current_step = state.current_step.prev();
            state.active_input_field = 0; // reset
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            trigger_disk_scan(state);
        }
        _ => {}
    }
}

pub fn trigger_disk_scan(state: &mut AppState) {
    state.disks.clear();
    
    // Jalankan lsblk untuk memindai disk fisik
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("lsblk").args(["-Jd", "-o", "NAME,SIZE,TYPE,MODEL"]).output() {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(lsblk_out) = serde_json::from_str::<LsblkOutput>(&stdout_str) {
                for dev in lsblk_out.blockdevices {
                    // Hanya masukkan device bertipe 'disk'
                    if dev.r#type == "disk" {
                        state.disks.push(DiskInfo {
                            name: dev.name,
                            size: dev.size,
                            type_name: dev.r#type,
                            model: dev.model,
                        });
                    }
                }
            }
        }
    }

    // Fallback/Simulasi untuk testing lokal non-linux
    if state.disks.is_empty() {
        state.disks = vec![
            DiskInfo {
                name: "nvme0n1".to_string(),
                size: "953.9G".to_string(),
                type_name: "disk".to_string(),
                model: Some("Samsung SSD 980 PRO 1TB".to_string()),
            },
            DiskInfo {
                name: "sda".to_string(),
                size: "447.1G".to_string(),
                type_name: "disk".to_string(),
                model: Some("Crucial CT480BX500SSD1".to_string()),
            },
        ];
    }

    if !state.disks.is_empty() && state.selected_disk.is_empty() {
        state.selected_disk = state.disks[0].name.clone();
    }
}

fn run_cfdisk(disk_path: &str) -> std::io::Result<()> {
    // 1. Matikan TUI terminal
    let _ = crate::tui::restore();

    // 2. Beri pesan ke user bahwa installer ditangguhkan sementara
    println!("\x1B[1;33m[Installer Ditangguhkan]\x1B[0m Membuka cfdisk untuk disk {}...", disk_path);
    println!("Gunakan antarmuka cfdisk untuk mempartisi disk Anda.");
    println!("Tekan [Write] untuk menyimpan partisi, lalu pilih [Quit] setelah selesai.");
    println!("Menjalankan: cfdisk {} ...", disk_path);
    
    // Tunggu sedikit agar pengguna bisa membaca
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // 3. Jalankan cfdisk secara sinkron
    let mut child = Command::new("cfdisk")
        .arg(disk_path)
        .spawn()?;
    
    let _status = child.wait()?;

    // 4. Nyalakan kembali TUI terminal
    let _ = crate::tui::init();
    
    Ok(())
}
