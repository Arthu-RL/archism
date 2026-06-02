use config::InstallConfig;
use constants::{LOCALES, TIMEZONES, KEYMAPS, DMS, GPUS};

#[derive(Clone, Copy, PartialEq)]
pub enum Step {
    Wifi,
    Configure,
    Review,
    Install,
    Success,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AppField {
    Disk,
    HorizontalLayout,
    Hostname,
    Username,
    Locale,
    Timezone,
    Keymap,
    SwapSize,
    Dm,
    Gpu,
}

pub struct AppState {
    pub step: Step,
    pub active_field: AppField,
    pub disks: Vec<String>,
    pub disk_idx: usize,
    pub hostname: String,
    pub username: String,
    pub locale_idx: usize,
    pub timezone_idx: usize,
    pub keymap_idx: usize,
    pub swap_size: u32,
    pub dm_idx: usize,
    pub gpu_idx: usize,
    pub logs: Vec<String>,
    pub progress_percent: u8,
    pub progress_stage: String,
    pub error_message: Option<String>,
    pub wifi_list: Vec<String>,
    pub wifi_idx: usize,
    pub wifi_password: String,
    pub wifi_input_mode: bool,
    pub wifi_status: String,
}

impl AppState {
    pub fn new(disks: Vec<String>) -> Self {
        Self {
            step: Step::Wifi,
            active_field: AppField::Disk,
            disks,
            disk_idx: 0,
            hostname: "archbox".to_string(),
            username: "".to_string(),
            locale_idx: 0,
            timezone_idx: 0,
            keymap_idx: 0,
            swap_size: 8,
            dm_idx: 0,
            gpu_idx: 0,
            logs: Vec::new(),
            progress_percent: 0,
            progress_stage: String::new(),
            error_message: None,
            wifi_list: vec!["Usa conexão com fio (Ethernet) / Pular".to_string()],
            wifi_idx: 0,
            wifi_password: String::new(),
            wifi_input_mode: false,
            wifi_status: "Aguardando ação...".to_string(),
        }
    }

    pub fn current_config(&self) -> InstallConfig {
        InstallConfig {
            disk: self.disks.get(self.disk_idx).cloned().unwrap_or_default(),
            hostname: self.hostname.clone(),
            username: self.username.clone(),
            locale: LOCALES[self.locale_idx].to_string(),
            timezone: TIMEZONES[self.timezone_idx].to_string(),
            keymap: KEYMAPS[self.keymap_idx].to_string(),
            dm: DMS[self.dm_idx].to_string(),
            gpu: GPUS[self.gpu_idx].to_string(),
            swap_size: self.swap_size,
        }
    }

    pub fn can_continue(&self) -> bool {
        !self.disks.is_empty()
            && !self.hostname.trim().is_empty()
            && self.username.trim().len() >= 2
    }
}