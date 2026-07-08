use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::state::{AppState, ShellOption, TerminalOption};

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
    let header = Paragraph::new("PILIHAN SHELL & TERMINAL")
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

    // Left Panel: Shell Option
    let shells = vec![ShellOption::ZshOhMyZsh, ShellOption::Fish];
    let shell_items: Vec<ListItem> = shells.iter().map(|s| {
        let prefix = if state.shell == *s { "● " } else { "○ " };
        ListItem::new(format!("{}{}", prefix, s))
    }).collect();

    let mut shell_list_state = ListState::default();
    let shell_idx = shells.iter().position(|&s| s == state.shell).unwrap_or(0);
    shell_list_state.select(Some(shell_idx));

    let shell_list = List::new(shell_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if state.active_input_field == 0 { Color::Cyan } else { Color::DarkGray }))
                .title(" 1. Pilih Default Shell "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(shell_list, main_layout[0], &mut shell_list_state);

    // Right Panel: Terminal Option
    let terminals = vec![TerminalOption::Kitty, TerminalOption::Alacritty];
    let terminal_items: Vec<ListItem> = terminals.iter().map(|t| {
        let prefix = if state.terminal == *t { "● " } else { "○ " };
        ListItem::new(format!("{}{}", prefix, t))
    }).collect();

    let mut terminal_list_state = ListState::default();
    let term_idx = terminals.iter().position(|&t| t == state.terminal).unwrap_or(0);
    terminal_list_state.select(Some(term_idx));

    let terminal_list = List::new(terminal_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if state.active_input_field == 1 { Color::Cyan } else { Color::DarkGray }))
                .title(" 2. Pilih Emulator Terminal "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(terminal_list, main_layout[1], &mut terminal_list_state);

    // Footer
    let help_text = format!(
        "Shell: {} | Terminal: {}\n[Tab]: Pindah Panel | [Panah Up/Down]: Pilih | [ENTER]: Lanjut | [ESC]: Kembali",
        state.shell,
        state.terminal
    );
    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            state.active_input_field = if state.active_input_field == 0 { 1 } else { 0 };
        }
        KeyCode::Up => {
            if state.active_input_field == 0 {
                let options = vec![ShellOption::ZshOhMyZsh, ShellOption::Fish];
                let current_idx = options.iter().position(|&s| s == state.shell).unwrap_or(0);
                let next_idx = if current_idx > 0 { current_idx - 1 } else { options.len() - 1 };
                state.shell = options[next_idx];
            } else {
                let options = vec![TerminalOption::Kitty, TerminalOption::Alacritty];
                let current_idx = options.iter().position(|&t| t == state.terminal).unwrap_or(0);
                let next_idx = if current_idx > 0 { current_idx - 1 } else { options.len() - 1 };
                state.terminal = options[next_idx];
            }
        }
        KeyCode::Down => {
            if state.active_input_field == 0 {
                let options = vec![ShellOption::ZshOhMyZsh, ShellOption::Fish];
                let current_idx = options.iter().position(|&s| s == state.shell).unwrap_or(0);
                let next_idx = if current_idx < options.len() - 1 { current_idx + 1 } else { 0 };
                state.shell = options[next_idx];
            } else {
                let options = vec![TerminalOption::Kitty, TerminalOption::Alacritty];
                let current_idx = options.iter().position(|&t| t == state.terminal).unwrap_or(0);
                let next_idx = if current_idx < options.len() - 1 { current_idx + 1 } else { 0 };
                state.terminal = options[next_idx];
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if state.active_input_field == 0 {
                state.active_input_field = 1;
            } else {
                state.current_step = state.current_step.next();
                state.active_input_field = 0; // reset
            }
        }
        KeyCode::Esc | KeyCode::Backspace => {
            state.current_step = state.current_step.prev();
            state.active_input_field = 0; // reset
        }
        _ => {}
    }
}
