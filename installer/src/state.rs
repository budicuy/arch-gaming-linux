use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Network,
    Partition,
    Desktop,
    Kernel,
    Shell,
    Gaming,
    User,
    Summary,
    Install,
}

impl Step {
    pub fn next(self) -> Self {
        match self {
            Step::Welcome => Step::Network,
            Step::Network => Step::Partition,
            Step::Partition => Step::Desktop,
            Step::Desktop => Step::Kernel,
            Step::Kernel => Step::Shell,
            Step::Shell => Step::Gaming,
            Step::Gaming => Step::User,
            Step::User => Step::Summary,
            Step::Summary => Step::Install,
            Step::Install => Step::Install,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Step::Welcome => Step::Welcome,
            Step::Network => Step::Welcome,
            Step::Partition => Step::Network,
            Step::Desktop => Step::Partition,
            Step::Kernel => Step::Desktop,
            Step::Shell => Step::Kernel,
            Step::Gaming => Step::Shell,
            Step::User => Step::Gaming,
            Step::Summary => Step::User,
            Step::Install => Step::Summary,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionType {
    AutoBtrfs,
    AutoExt4,
    ManualCfdisk,
}

impl std::fmt::Display for PartitionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionType::AutoBtrfs => write!(f, "Otomatis: BTRFS"),
            PartitionType::AutoExt4 => write!(f, "Otomatis: EXT4"),
            PartitionType::ManualCfdisk => write!(f, "Manual: cfdisk"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopEnv {
    Hyprland,
    Sway,
    KdePlasma,
    Gnome,
    Xfce,
    GamescopeOnly,
}

impl std::fmt::Display for DesktopEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DesktopEnv::Hyprland => write!(f, "Hyprland (Modern Wayland Compositor)"),
            DesktopEnv::Sway => write!(f, "Sway (i3-like Wayland Compositor)"),
            DesktopEnv::KdePlasma => write!(f, "KDE Plasma (Feature-rich Modern Desktop)"),
            DesktopEnv::Gnome => write!(f, "GNOME (Polished Modern Desktop)"),
            DesktopEnv::Xfce => write!(f, "XFCE (Classic Lightweight Desktop)"),
            DesktopEnv::GamescopeOnly => write!(f, "Gamescope Only (Direct-to-Steam Micro-Compositor)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelOption {
    LinuxTkgZen,
}

impl std::fmt::Display for KernelOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KernelOption::LinuxTkgZen => write!(f, "linux-tkg-zen (Custom Gaming Optimization)"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellOption {
    ZshOhMyZsh,
    Fish,
}

impl std::fmt::Display for ShellOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellOption::ZshOhMyZsh => write!(f, "Zsh + OhMyZsh"),
            ShellOption::Fish => write!(f, "Fish Shell"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalOption {
    Kitty,
    Alacritty,
}

impl std::fmt::Display for TerminalOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TerminalOption::Kitty => write!(f, "Kitty (GPU Accelerated, Rich Features)"),
            TerminalOption::Alacritty => write!(f, "Alacritty (Simple & Ultra Fast)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub size: String,
    pub type_name: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsblkOutput {
    pub blockdevices: Vec<LsblkDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsblkDevice {
    pub name: String,
    pub size: String,
    pub r#type: String,
    pub model: Option<String>,
}

pub struct AppState {
    pub current_step: Step,
    pub network_ssid: String,
    pub network_pass: String,
    pub selected_disk: String,
    pub partition_type: PartitionType,
    pub desktop: DesktopEnv,
    pub kernel: KernelOption,
    pub shell: ShellOption,
    pub terminal: TerminalOption,
    pub install_steam: bool,
    pub install_mangohud: bool,
    pub install_gamemode: bool,
    pub install_protonup: bool,
    pub install_wine: bool,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    // UI state
    pub active_input_field: usize, // steps with multiple text fields
    pub list_selected: usize,
    pub ssids: Vec<String>,
    pub scanning_wifi: bool,
    pub disks: Vec<DiskInfo>,
    pub install_progress: f64,
    pub install_status: String,
    pub install_logs: Vec<String>,
    pub is_installing: bool,
    pub install_completed: bool,
    pub error_message: Option<String>,
    pub show_wifi_password_popup: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_step: Step::Welcome,
            network_ssid: String::new(),
            network_pass: String::new(),
            selected_disk: String::new(),
            partition_type: PartitionType::AutoBtrfs,
            desktop: DesktopEnv::Hyprland,
            kernel: KernelOption::LinuxTkgZen,
            shell: ShellOption::ZshOhMyZsh,
            terminal: TerminalOption::Kitty,
            install_steam: true,
            install_mangohud: true,
            install_gamemode: true,
            install_protonup: true,
            install_wine: true,
            hostname: "arch-gaming".to_string(),
            username: "gaming".to_string(),
            password: String::new(),
            confirm_password: String::new(),
            active_input_field: 0,
            list_selected: 0,
            ssids: Vec::new(),
            scanning_wifi: false,
            disks: Vec::new(),
            install_progress: 0.0,
            install_status: "Menunggu persetujuan...".to_string(),
            install_logs: Vec::new(),
            is_installing: false,
            install_completed: false,
            error_message: None,
            show_wifi_password_popup: false,
        }
    }
}
