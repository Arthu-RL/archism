use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InstallConfig {
    pub disk: String,
    pub hostname: String,
    pub username: String,
    pub locale: String,
    pub timezone: String,
    pub keymap: String,
    pub dm: String,
    pub gpu: String,
    pub swap_size: u32,
}

pub fn validate_safe(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("O campo '{}' não pode ser vazio.", field));
    }
    let unsafe_chars = ['$', '`', ';', '&', '|', '\n', '\r', '"', '\'', '\\', '<', '>'];
    if let Some(c) = value.chars().find(|c| unsafe_chars.contains(c)) {
        return Err(format!("Caractere inválido '{}' encontrado em '{}'.", c, field));
    }
    Ok(())
}

pub fn validate_config(cfg: &InstallConfig) -> Result<(), String> {
    if !cfg.disk.starts_with("/dev/") {
        return Err("Caminho de disco inválido: deve iniciar com /dev/.".to_string());
    }
    if cfg.swap_size > 64 {
        return Err("O tamanho de swap não pode exceder 64 GB.".to_string());
    }
    validate_safe(&cfg.hostname, "hostname")?;
    validate_safe(&cfg.username, "username")?;
    validate_safe(&cfg.locale, "locale")?;
    validate_safe(&cfg.timezone, "timezone")?;
    validate_safe(&cfg.keymap, "keymap")?;
    validate_safe(&cfg.dm, "display manager")?;
    Ok(())
}